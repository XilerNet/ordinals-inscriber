use crate::{db::PaymentRepository, subcommand::wallet::get_change_address};

use {
  super::*,
  crate::{db::Repository, subcommand::wallet::transaction_builder::Target, wallet::Wallet},
  bitcoin::{
    blockdata::{opcodes, script},
    key::PrivateKey,
    key::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    locktime::absolute::LockTime,
    policy::MAX_STANDARD_TX_WEIGHT,
    secp256k1::{self, constants::SCHNORR_SIGNATURE_SIZE, rand, Secp256k1, XOnlyPublicKey},
    sighash::{Prevouts, SighashCache, TapSighashType},
    taproot::Signature,
    taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
    ScriptBuf, Witness,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
  bitcoincore_rpc::Client,
  std::collections::BTreeSet,
};

const INSCRIPTION_TYPE: &'static str = "text/plain";

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub commit: Txid,
  pub inscription: InscriptionId,
  pub parent: Option<InscriptionId>,
  pub reveal: Txid,
  pub total_fees: u64,
}

#[derive(Clone)]
struct ParentInfo {
  destination: Address,
  location: SatPoint,
  tx_out: TxOut,
}

#[derive(Debug, Parser)]
pub(crate) struct Inscriber {}

impl Inscriber {
  async fn inscriptions_inscriber(self, options: Options) {
    let repository = Repository::new().await;
    let index = Index::open(&options).unwrap();
    let client = options
      .bitcoin_rpc_client_for_wallet_command(false)
      .unwrap();

    loop {
      if let Err(err) = index.update() {
        println!("error: {:?}", err);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        continue;
      }

      let to_complete_inscription_ids = repository.get_to_be_completed_payments().await;
      println!(
        "to_complete_inscriptions: {:?}",
        to_complete_inscription_ids
      );

      if to_complete_inscription_ids.is_err() {
        println!("error: {:?}", to_complete_inscription_ids);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        continue;
      }

      let to_complete_inscription_ids = to_complete_inscription_ids.unwrap();
      let utxos = index
        .get_unspent_outputs(Wallet::load(&options).unwrap())
        .unwrap();

      for id in to_complete_inscription_ids {
        println!("inscribing inscription: {:?}", id);
        let res = repository.get_payment_inscriptions_content(&id).await;

        if res.is_err() {
          println!("error: {:?}", res);
          continue;
        }

        let res = res.unwrap();

        if res.is_none() {
          println!("error: {:?}", res);
          continue;
        }

        let contents = res.unwrap();

        for (target, content) in contents {
          if let Some(limit) = options.chain().inscription_content_size_limit() {
            if content.len() > limit {
              println!("content too large: {:?}: {:?}", id, target);
              continue;
            }
          }

          println!("target: {:?}", target);
          let inscription = Inscription::new(Some(INSCRIPTION_TYPE.into()), Some(content.into()));
          println!("inscription: {:?}", inscription);

          let commit_tx_change = [
            get_change_address(&client, &options).unwrap(),
            get_change_address(&client, &options).unwrap(),
          ];

          let destination = Address::<NetworkUnchecked>::from_str(&target);
          if destination.is_err() {
            println!("error: {:?}", destination);
            continue;
          }

          let reveal_tx_destination = destination
            .unwrap()
            .require_network(options.chain().network());

          if reveal_tx_destination.is_err() {
            println!("error: {:?}", reveal_tx_destination);
            continue;
          }
          let reveal_tx_destination = reveal_tx_destination.unwrap();
        }
      }

      println!("inscribing inscriptions");
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
  }

  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    color_eyre::install().ok();
    dotenv::dotenv().ok();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(self.inscriptions_inscriber(options));

    // let inscription =
    //   Inscription::from_file(options.chain(), &self.file, self.parent, self.metaprotocol)?;
    //
    // let index = Index::open(&options)?;
    // index.update()?;
    //
    // let client = options.bitcoin_rpc_client_for_wallet_command(false)?;
    //
    // let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;
    //
    // let inscriptions = index.get_inscriptions(utxos.clone())?;
    //
    // let commit_tx_change = [
    //   get_change_address(&client, &options)?,
    //   get_change_address(&client, &options)?,
    // ];
    //
    // let reveal_tx_destination = match self.destination {
    //   Some(address) => address.require_network(options.chain().network())?,
    //   None => get_change_address(&client, &options)?,
    // };
    //
    // let parent_info = if let Some(parent_id) = self.parent {
    //   if let Some(satpoint) = index.get_inscription_satpoint_by_id(parent_id)? {
    //     if !utxos.contains_key(&satpoint.outpoint) {
    //       return Err(anyhow!(format!("parent {parent_id} not in wallet")));
    //     }
    //
    //     Some(ParentInfo {
    //       destination: get_change_address(&client, &options)?,
    //       location: satpoint,
    //       tx_out: index
    //         .get_transaction(satpoint.outpoint.txid)?
    //         .expect("parent transaction not found in index")
    //         .output
    //         .into_iter()
    //         .nth(satpoint.outpoint.vout.try_into().unwrap())
    //         .expect("current transaction output"),
    //     })
    //   } else {
    //     return Err(anyhow!(format!("parent {parent_id} does not exist")));
    //   }
    // } else {
    //   None
    // };
    //
    // let (commit_tx, reveal_tx, recovery_key_pair, total_fees) =
    //   Inscribe::create_inscription_transactions(
    //     self.satpoint,
    //     parent_info,
    //     inscription,
    //     inscriptions,
    //     options.chain().network(),
    //     utxos,
    //     commit_tx_change,
    //     reveal_tx_destination,
    //     self.commit_fee_rate.unwrap_or(self.fee_rate),
    //     self.fee_rate,
    //     self.no_limit,
    //     self.reinscribe,
    //     match self.postage {
    //       Some(postage) => postage,
    //       _ => TransactionBuilder::TARGET_POSTAGE,
    //     },
    //   )?;
    //
    // if self.dry_run {
    //   return Ok(Box::new(Output {
    //     commit: commit_tx.txid(),
    //     reveal: reveal_tx.txid(),
    //     inscription: InscriptionId {
    //       txid: reveal_tx.txid(),
    //       index: 0,
    //     },
    //     parent: self.parent,
    //     total_fees,
    //   }));
    // }
    //
    // let signed_commit_tx = client
    //   .sign_raw_transaction_with_wallet(&commit_tx, None, None)?
    //   .hex;
    //
    // let signed_reveal_tx = if self.parent.is_some() {
    //   client
    //     .sign_raw_transaction_with_wallet(
    //       &reveal_tx,
    //       Some(
    //         &commit_tx
    //           .output
    //           .iter()
    //           .enumerate()
    //           .map(|(vout, output)| SignRawTransactionInput {
    //             txid: commit_tx.txid(),
    //             vout: vout.try_into().unwrap(),
    //             script_pub_key: output.script_pubkey.clone(),
    //             redeem_script: None,
    //             amount: Some(Amount::from_sat(output.value)),
    //           })
    //           .collect::<Vec<SignRawTransactionInput>>(),
    //       ),
    //       None,
    //     )?
    //     .hex
    // } else {
    //   bitcoin::consensus::encode::serialize(&reveal_tx)
    // };
    //
    // if !self.no_backup {
    //   Inscribe::backup_recovery_key(&client, recovery_key_pair, options.chain().network())?;
    // }
    //
    // let commit = client.send_raw_transaction(&signed_commit_tx)?;
    //
    // let reveal = client
    //   .send_raw_transaction(&signed_reveal_tx)
    //   .context("Failed to send reveal transaction")?;
    //
    // Ok(Box::new(Output {
    //   commit,
    //   reveal,
    //   parent: self.parent,
    //   inscription: InscriptionId {
    //     txid: reveal,
    //     index: 0,
    //   },
    //   total_fees,
    // }))
    todo!();
  }

  fn create_inscription_transactions(
    satpoint: Option<SatPoint>,
    parent_info: Option<ParentInfo>,
    inscription: Inscription,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    network: Network,
    mut utxos: BTreeMap<OutPoint, Amount>,
    change: [Address; 2],
    destination: Address,
    commit_fee_rate: FeeRate,
    reveal_fee_rate: FeeRate,
    no_limit: bool,
    reinscribe: bool,
    postage: Amount,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair, u64)> {
    let satpoint = if let Some(satpoint) = satpoint {
      satpoint
    } else {
      let inscribed_utxos = inscriptions
        .keys()
        .map(|satpoint| satpoint.outpoint)
        .collect::<BTreeSet<OutPoint>>();

      utxos
        .keys()
        .find(|outpoint| !inscribed_utxos.contains(outpoint))
        .map(|outpoint| SatPoint {
          outpoint: *outpoint,
          offset: 0,
        })
        .ok_or_else(|| anyhow!("wallet contains no cardinal utxos"))?
    };

    let mut reinscription = false;

    for (inscribed_satpoint, inscription_id) in &inscriptions {
      if *inscribed_satpoint == satpoint {
        reinscription = true;
        if reinscribe {
          continue;
        } else {
          return Err(anyhow!("sat at {} already inscribed", satpoint));
        }
      }

      if inscribed_satpoint.outpoint == satpoint.outpoint {
        return Err(anyhow!(
          "utxo {} already inscribed with inscription {inscription_id} on sat {inscribed_satpoint}",
          satpoint.outpoint,
        ));
      }
    }

    if reinscribe && !reinscription {
      return Err(anyhow!(
        "reinscribe flag set but this would not be a reinscription"
      ));
    }

    let secp256k1 = Secp256k1::new();
    let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let reveal_script = inscription.append_reveal_script(
      ScriptBuf::builder()
        .push_slice(public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG),
    );

    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, reveal_script.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
      .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);

    let mut inputs = vec![OutPoint::null()];
    let mut outputs = vec![TxOut {
      script_pubkey: destination.script_pubkey(),
      value: 0,
    }];

    if let Some(ParentInfo {
      location,
      destination,
      tx_out,
    }) = parent_info.clone()
    {
      inputs.insert(0, location.outpoint);
      outputs.insert(
        0,
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: tx_out.value,
        },
      );
    }

    let commit_input = if parent_info.is_some() { 1 } else { 0 };

    let (_, reveal_fee) = Self::build_reveal_transaction(
      &control_block,
      reveal_fee_rate,
      inputs.clone(),
      commit_input,
      outputs.clone(),
      &reveal_script,
    );

    let unsigned_commit_tx = TransactionBuilder::new(
      satpoint,
      inscriptions,
      utxos.clone(),
      commit_tx_address.clone(),
      change,
      commit_fee_rate,
      Target::Value(reveal_fee + postage),
    )
    .build_transaction()?;

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find sat commit/inscription output");

    inputs[commit_input] = OutPoint {
      txid: unsigned_commit_tx.txid(),
      vout: vout.try_into().unwrap(),
    };

    outputs[commit_input] = TxOut {
      script_pubkey: destination.script_pubkey(),
      value: output.value,
    };

    let (mut reveal_tx, fee) = Self::build_reveal_transaction(
      &control_block,
      reveal_fee_rate,
      inputs,
      commit_input,
      outputs.clone(),
      &reveal_script,
    );

    reveal_tx.output[commit_input].value = reveal_tx.output[commit_input]
      .value
      .checked_sub(fee.to_sat())
      .context("commit transaction output value insufficient to pay transaction fee")?;

    if reveal_tx.output[commit_input].value
      < reveal_tx.output[commit_input]
        .script_pubkey
        .dust_value()
        .to_sat()
    {
      bail!("commit transaction output would be dust");
    }

    let mut prevouts = vec![unsigned_commit_tx.output[0].clone()];

    if let Some(parent_info) = parent_info {
      prevouts.insert(0, parent_info.tx_out);
    }

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);

    let sighash = sighash_cache
      .taproot_script_spend_signature_hash(
        commit_input,
        &Prevouts::All(&prevouts),
        TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
        TapSighashType::Default,
      )
      .expect("signature hash should compute");

    let sig = secp256k1.sign_schnorr(
      &secp256k1::Message::from_slice(sighash.as_ref())
        .expect("should be cryptographically secure hash"),
      &key_pair,
    );

    let witness = sighash_cache
      .witness_mut(commit_input)
      .expect("getting mutable witness reference should work");

    witness.push(
      Signature {
        sig,
        hash_ty: TapSighashType::Default,
      }
      .to_vec(),
    );

    witness.push(reveal_script);
    witness.push(&control_block.serialize());

    let recovery_key_pair = key_pair.tap_tweak(&secp256k1, taproot_spend_info.merkle_root());

    let (x_only_pub_key, _parity) = recovery_key_pair.to_inner().x_only_public_key();
    assert_eq!(
      Address::p2tr_tweaked(
        TweakedPublicKey::dangerous_assume_tweaked(x_only_pub_key),
        network,
      ),
      commit_tx_address
    );

    let reveal_weight = reveal_tx.weight();

    if !no_limit && reveal_weight > bitcoin::Weight::from_wu(MAX_STANDARD_TX_WEIGHT.into()) {
      bail!(
        "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
      );
    }

    utxos.insert(
      reveal_tx.input[commit_input].previous_output,
      Amount::from_sat(
        unsigned_commit_tx.output[reveal_tx.input[commit_input].previous_output.vout as usize]
          .value,
      ),
    );

    let total_fees =
      Self::calculate_fee(&unsigned_commit_tx, &utxos) + Self::calculate_fee(&reveal_tx, &utxos);

    Ok((unsigned_commit_tx, reveal_tx, recovery_key_pair, total_fees))
  }

  fn build_reveal_transaction(
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    inputs: Vec<OutPoint>,
    commit_input_index: usize,
    outputs: Vec<TxOut>,
    script: &Script,
  ) -> (Transaction, Amount) {
    let reveal_tx = Transaction {
      input: inputs
        .iter()
        .map(|outpoint| TxIn {
          previous_output: *outpoint,
          script_sig: script::Builder::new().into_script(),
          witness: Witness::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        })
        .collect(),
      output: outputs,
      lock_time: LockTime::ZERO,
      version: 1,
    };

    let fee = {
      let mut reveal_tx = reveal_tx.clone();

      for (current_index, txin) in reveal_tx.input.iter_mut().enumerate() {
        // add dummy inscription witness for reveal input/commit output
        if current_index == commit_input_index {
          txin.witness.push(
            Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
              .unwrap()
              .to_vec(),
          );
          txin.witness.push(script);
          txin.witness.push(&control_block.serialize());
        } else {
          txin.witness = Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]);
        }
      }

      fee_rate.fee(reveal_tx.vsize())
    };

    (reveal_tx, fee)
  }

  fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, Amount>) -> u64 {
    tx.input
      .iter()
      .map(|txin| utxos.get(&txin.previous_output).unwrap().to_sat())
      .sum::<u64>()
      .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
      .unwrap()
  }

  fn backup_recovery_key(
    client: &Client,
    recovery_key_pair: TweakedKeyPair,
    network: Network,
  ) -> Result {
    let recovery_private_key = PrivateKey::new(recovery_key_pair.to_inner().secret_key(), network);

    let info = client.get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

    let response = client.import_descriptors(ImportDescriptors {
      descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
      timestamp: Timestamp::Now,
      active: Some(false),
      range: None,
      next_index: None,
      internal: Some(false),
      label: Some("commit tx recovery key".to_string()),
    })?;

    for result in response {
      if !result.success {
        return Err(anyhow!("commit tx recovery key import failed"));
      }
    }

    Ok(())
  }
}