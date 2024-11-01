use std::ops::Neg;

use ark_ec::{msm::VariableBaseMSM, AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{Field, PrimeField, ToBytes};

use crate::{
    param::{ProverParam, VerifierParam},
    Commitment, CommitmentScheme,
};

impl<E: PairingEngine, const N: usize> CommitmentScheme for Commitment<E, N> {
    type ProverParam = ProverParam<E, N>;
    type VerifierParam = VerifierParam<E, N>;
    type MessageUnit = E::Fr;
    type Commitment = Self;
    type Witness = E::G1Projective;

    /// Commit to a list of inputs with prover parameters
    fn commit(pp: &Self::ProverParam, inputs: &[Self::MessageUnit]) -> Self {
        assert!(inputs.len() <= N);

        let scalars: Vec<<E::Fr as PrimeField>::BigInt> =
            inputs.iter().map(|x| x.into_repr()).collect();

        Self {
            inner: VariableBaseMSM::multi_scalar_mul(&pp.g[0..inputs.len()], scalars.as_ref()),
        }
    }

    /// Open an input at a given position
    fn open(pp: &Self::ProverParam, inputs: &[Self::MessageUnit], pos: usize) -> Self::Witness {
        assert!(inputs.len() <= N);

        let scalars: Vec<<E::Fr as PrimeField>::BigInt> =
            inputs.iter().map(|x| x.into_repr()).collect();
        VariableBaseMSM::multi_scalar_mul(
            pp.g[N - pos..N - pos + inputs.len()].as_ref(),
            scalars.as_ref(),
        )
    }

    /// Verify the input/witness pair is correct
    fn verify(
        &self,
        vp: &Self::VerifierParam,
        input: &Self::MessageUnit,
        pos: usize,
        witness: &Self::Witness,
    ) -> bool {
        let input_inverse = input.inverse().unwrap();

        let com = self.inner.mul(&input_inverse.into_repr()).into_affine();
        let proof = witness.mul(input_inverse.neg().into_repr()).into_affine();
        let pairing_prod_inputs = [
            (com.into(), vp.h[N - pos - 1].into()),
            (proof.into(), E::G2Affine::prime_subgroup_generator().into()),
        ];
        E::product_of_pairings(pairing_prod_inputs.iter()) == vp.t
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        self.inner.write(&mut buffer).unwrap();

        buffer
    }
}
