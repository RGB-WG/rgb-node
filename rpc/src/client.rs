// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::thread::sleep;
use std::time::Duration;

use colored::Colorize;
use internet2::addr::ServiceAddr;
use internet2::ZmqSocketType;
use microservices::esb::{self, BusId};

use crate::{BusMsg, ClientId, Error, OptionDetails, RpcMsg, ServiceId};

// We have just a single service bus (RPC), so we can use any id
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, Display)]
#[display("RGBRPC")]
struct RpcBus;

impl BusId for RpcBus {
    type Address = ServiceId;
}

type Bus = esb::EndpointList<RpcBus>;

#[repr(C)]
pub struct Client {
    identity: ClientId,
    response_queue: Vec<RpcMsg>,
    esb: esb::Controller<RpcBus, BusMsg, Handler>,
}

impl Client {
    pub fn with(connect_addr: ServiceAddr) -> Result<Self, Error> {
        use rgb::secp256k1zkp::rand;

        debug!("RPC socket {}", connect_addr);

        debug!("Setting up RPC client...");
        let identity = rand::random();
        let bus_config = esb::BusConfig::with_addr(
            connect_addr,
            ZmqSocketType::RouterConnect,
            Some(ServiceId::router()),
        );
        let esb = esb::Controller::with(
            map! {
                RpcBus => bus_config
            },
            Handler {
                identity: ServiceId::Client(identity),
            },
        )?;

        // We have to sleep in order for ZMQ to bootstrap
        sleep(Duration::from_secs_f32(0.1));

        Ok(Self {
            identity,
            response_queue: empty!(),
            esb,
        })
    }

    pub fn identity(&self) -> ClientId { self.identity }

    pub fn request(&mut self, daemon: ServiceId, req: RpcMsg) -> Result<(), Error> {
        debug!("Executing {}", req);
        self.esb.send_to(RpcBus, daemon, BusMsg::Rpc(req))?;
        Ok(())
    }

    pub fn response(&mut self) -> Result<RpcMsg, Error> {
        if self.response_queue.is_empty() {
            for poll in self.esb.recv_poll()? {
                match poll.request {
                    BusMsg::Rpc(msg) => self.response_queue.push(msg),
                }
            }
        }
        Ok(self.response_queue.pop().expect("We always have at least one element"))
    }

    pub fn report_failure(&mut self) -> Result<RpcMsg, Error> {
        match self.response()? {
            RpcMsg::Failure(fail) => {
                eprintln!("{}: {}", "Request failure".bright_red(), fail.to_string().red());
                Err(Error::Server(fail.into()))
            }
            resp => Ok(resp),
        }
    }

    pub fn report_response(&mut self) -> Result<(), Error> {
        let resp = self.report_failure()?;
        println!("{:#}", resp);
        Ok(())
    }

    pub fn report_progress(&mut self) -> Result<usize, Error> {
        let mut counter = 0;
        let mut finished = false;
        while !finished {
            finished = true;
            counter += 1;
            match self.report_failure()? {
                // Failure is already covered by `report_response()`
                RpcMsg::Progress(info) => {
                    println!("{}", info);
                    finished = false;
                }
                RpcMsg::Success(OptionDetails(Some(info))) => {
                    println!("{}{}", "Success: ".bright_green(), info);
                }
                RpcMsg::Success(OptionDetails(None)) => {
                    println!("{}", "Success".bright_green());
                }
                RpcMsg::UnresolvedTxids(txids) => {
                    eprintln!(
                        "{}: some of the transactions can't be resolved",
                        "Warning".bright_yellow()
                    );
                    for txid in txids {
                        println!("{}", txid);
                    }
                }
                RpcMsg::Invalid(status) => {
                    eprintln!("{}: consignment is invalid", "Error".bright_red());
                    #[cfg(feature = "serde")]
                    eprintln!("{}", serde_yaml::to_string(&status).unwrap());
                    #[cfg(not(feature = "serde"))]
                    eprintln!("{:#?}", status);
                }
                other => {
                    eprintln!(
                        "{}: {}",
                        "Unexpected message".bright_yellow(),
                        other.to_string().yellow()
                    );
                    return Err(Error::Other(s!("Unexpected server response")));
                }
            }
        }
        Ok(counter)
    }
}

pub struct Handler {
    identity: ServiceId,
}

impl esb::Handler<RpcBus> for Handler {
    type Request = BusMsg;
    type Error = Error;

    fn identity(&self) -> ServiceId { self.identity.clone() }

    fn handle(
        &mut self,
        _: &mut Bus,
        _: RpcBus,
        _: ServiceId,
        _: BusMsg,
    ) -> Result<(), Self::Error> {
        // Cli does not receive replies for now
        Ok(())
    }

    fn handle_err(&mut self, _: &mut Bus, err: esb::Error<ServiceId>) -> Result<(), Self::Error> {
        // We simply propagate the error since it already has been reported
        Err(err.into())
    }
}
