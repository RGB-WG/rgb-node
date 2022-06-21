// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use rgb::{validation, ConsignmentId, Contract, StateTransfer, Validity};

/// RPC API requests over CTL message bus between RGB Node daemons.
#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[non_exhaustive]
pub enum CtlMsg {
    #[display("hello()")]
    Hello,

    #[display("process_contract(...)")]
    ProcessContract(Contract),

    #[display("process_transfer(...)")]
    ProcessTransfer(StateTransfer),

    #[display("validity(...)")]
    Validity(ValidityReport),
}

#[derive(Clone, Debug, Default, StrictEncode, StrictDecode)]
pub struct ValidityReport {
    pub consignment_id: ConsignmentId,
    pub status: validation::Status,
}
