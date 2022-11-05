use crate::lib::job_negotiator::JobNegotiator;
use roles_logic_sv2::{
    handlers::{job_negotiation::ParseServerJobNegotiationMessages, SendTo_},
    job_negotiation_sv2::{
        AllocateMiningJobTokenSuccess, CommitMiningJob, CommitMiningJobError,
        CommitMiningJobSuccess, IdentifyTransactions, IdentifyTransactionsSuccess,
        ProvideMissingTransactions, ProvideMissingTransactionsSuccess,
    },
    parsers::JobNegotiation,
};
use tracing::info;
pub type SendTo = SendTo_<JobNegotiation<'static>, ()>;
use roles_logic_sv2::errors::Error;
use std::convert::TryInto;

impl ParseServerJobNegotiationMessages for JobNegotiator {
    fn allocate_mining_job_token_success(
        &mut self,
        message: AllocateMiningJobTokenSuccess,
    ) -> Result<SendTo, Error> {
        let coinbase_output_max_additional_size = message.coinbase_output_max_additional_size;

        let new_template = self.last_new_template.as_ref().unwrap();

        let message_commit_mining_job = CommitMiningJob {
            request_id: message.request_id,
            mining_job_token: message.mining_job_token.into_static(),
            version: 2,
            coinbase_tx_version: new_template.clone().coinbase_tx_version,
            coinbase_prefix: new_template.clone().coinbase_prefix,
            coinbase_tx_input_n_sequence: new_template.clone().coinbase_tx_input_sequence,
            coinbase_tx_value_remaining: new_template.clone().coinbase_tx_value_remaining,
            coinbase_tx_outputs: new_template.clone().coinbase_tx_outputs,
            coinbase_tx_locktime: new_template.clone().coinbase_tx_locktime,
            min_extranonce_size: 0,
            tx_short_hash_nonce: 0,
            /// Only for MVP2: must be filled with right values for production,
            /// this values are needed for block propagation
            tx_short_hash_list: vec![].try_into().unwrap(),
            tx_hash_list_hash: [0; 32].try_into().unwrap(),
            excess_data: vec![].try_into().unwrap(),
        };
        let commit_mining_job = JobNegotiation::CommitMiningJob(message_commit_mining_job);
        println!("Send commit mining job to pool: {:?}", commit_mining_job);
        Ok(SendTo::Respond(commit_mining_job))
    }

    fn commit_mining_job_success(
        &mut self,
        message: CommitMiningJobSuccess,
    ) -> Result<SendTo, Error> {
        info!("MVP2 ENDS HERE");
        Ok(SendTo::None(None))
    }

    fn commit_mining_job_error(&mut self, message: CommitMiningJobError) -> Result<SendTo, Error> {
        todo!();
    }

    fn identify_transactions(&mut self, message: IdentifyTransactions) -> Result<SendTo, Error> {
        let message_identify_transactions = IdentifyTransactionsSuccess {
            request_id: message.request_id,
            tx_hash_list: todo!(),
        };
        let message_enum =
            JobNegotiation::IdentifyTransactionsSuccess(message_identify_transactions);
        Ok(SendTo::Respond(message_enum))
    }

    fn provide_missing_transactions(
        &mut self,
        message: ProvideMissingTransactions,
    ) -> Result<SendTo, Error> {
        let message_provide_missing_transactions = ProvideMissingTransactionsSuccess {
            request_id: message.request_id,
            transaction_list: todo!(),
        };
        let message_enum =
            JobNegotiation::ProvideMissingTransactionsSuccess(message_provide_missing_transactions);
        Ok(SendTo::Respond(message_enum))
    }

    fn handle_message_job_negotiation(
        self_: std::sync::Arc<roles_logic_sv2::utils::Mutex<Self>>,
        message_type: u8,
        payload: &mut [u8],
    ) -> Result<SendTo, Error> {
        // Is ok to unwrap a safe_lock result
        match (message_type, payload).try_into() {
            Ok(JobNegotiation::AllocateMiningJobTokenSuccess(message)) => self_
                .safe_lock(|x| x.allocate_mining_job_token_success(message))
                .unwrap(),
            Ok(JobNegotiation::CommitMiningJobSuccess(message)) => self_
                .safe_lock(|x| x.commit_mining_job_success(message))
                .unwrap(),
            Ok(JobNegotiation::CommitMiningJobError(message)) => self_
                .safe_lock(|x| x.commit_mining_job_error(message))
                .unwrap(),
            Ok(JobNegotiation::IdentifyTransactions(message)) => self_
                .safe_lock(|x| x.identify_transactions(message))
                .unwrap(),
            Ok(JobNegotiation::ProvideMissingTransactions(message)) => self_
                .safe_lock(|x| x.provide_missing_transactions(message))
                .unwrap(),
            Ok(JobNegotiation::AllocateMiningJobToken(_)) => Err(Error::UnexpectedMessage),
            Ok(JobNegotiation::CommitMiningJob(_)) => Err(Error::UnexpectedMessage),
            Ok(JobNegotiation::IdentifyTransactionsSuccess(_)) => Err(Error::UnexpectedMessage),
            Ok(JobNegotiation::ProvideMissingTransactionsSuccess(_)) => {
                Err(Error::UnexpectedMessage)
            }
            Err(e) => Err(e),
        }
    }
}
