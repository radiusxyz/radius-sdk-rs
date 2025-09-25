use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

use super::{DkgValidationContractInstance, DkgValidationServiceResult};
use crate::DkgValidationServiceError;

#[derive(Debug, Clone)]
pub struct Publisher<P> {
    contract: DkgValidationContractInstance<P>,
}

impl<P: Provider> Publisher<P> {
    pub fn new(contract: DkgValidationContractInstance<P>) -> Self {
        Self { contract }
    }

    pub async fn is_solver(&self, who: Address) -> DkgValidationServiceResult<bool> {
        self.contract
            .isSolver(who)
            .call()
            .await
            .map_err(|e| e.into())
    }

    pub async fn is_operator(&self, who: Address) -> DkgValidationServiceResult<bool> {
        self.contract
            .isActiveCommittee(who)
            .call()
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_session_duration(&self) -> DkgValidationServiceResult<u64> {
        self.contract
            .getSessionDuration()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .try_into()
            .map_err(|_| DkgValidationServiceError::ConversionError)
    }

    pub async fn get_collecting_duration(&self) -> DkgValidationServiceResult<u64> {
        self.contract
            .getCollectingDuration()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .try_into()
            .map_err(|_| DkgValidationServiceError::ConversionError)
    }

    pub async fn get_threshold(&self) -> DkgValidationServiceResult<u16> {
        self.contract
            .getMinimumKeyThreshold()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .try_into()
            .map_err(|_| DkgValidationServiceError::ConversionError)
    }

    pub async fn update_trusted_setup(
        &self,
        trusted_setup: Vec<u8>,
    ) -> DkgValidationServiceResult<()> {
        let _ = self
            .contract
            .updateActiveTrustedSetup(trusted_setup.into())
            .gas(15_000_000)
            .gas_price(20000000000)
            .send()
            .await;
        Ok(())
    }

    pub async fn get_session_per_round(&self) -> DkgValidationServiceResult<u64> {
        self.contract
            .getSessionsPerRound()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .try_into()
            .map_err(|_| DkgValidationServiceError::ConversionError)
    }

    pub async fn get_active_trusted_setup(&self) -> DkgValidationServiceResult<Vec<u8>> {
        Ok(self
            .contract
            .getActiveTrustedSetup()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .to_vec())
    }

    pub async fn get_solver_info(&self) -> DkgValidationServiceResult<(Address, String, String)> {
        let res = self
            .contract
            .getSolverInfo()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?;
        Ok((res.currentSolver, res.clusterRpcUrl, res.externalRpcUrl))
    }

    pub async fn get_active_operators(
        &self,
    ) -> DkgValidationServiceResult<Vec<(Address, String, String)>> {
        Ok(self
            .contract
            .getActiveCommitteeList()
            .call()
            .await
            .map_err(|e| DkgValidationServiceError::ContractError(e))?
            .into_iter()
            .map(|c| (c.account, c.clusterRpcUrl, c.externalRpcUrl))
            .collect::<Vec<_>>())
    }

    pub async fn is_ready(&self, threshold: u16) -> DkgValidationServiceResult<bool> {
        Ok(self.get_active_operators().await?.len() >= threshold as usize)
    }

    pub async fn create_new_task(&self, session_id: u64, task: Vec<u8>) {
        let _ = self
            .contract
            .createNewTask(task.into(), U256::from(session_id))
            .send()
            .await;
    }

    pub async fn respond_task(&self, round: u64, session_id: u64) {
        let _ = self
            .contract
            .respondToTask(U256::from(round), U256::from(session_id))
            .send()
            .await;
    }
}
