use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_void};

use log::info;

use serde::Deserialize;

use rgb::lnpbp::bitcoin::OutPoint;

use rgb::lnpbp::bp;
use rgb::lnpbp::lnp::transport::zmq::{SocketLocator, UrlError};
use rgb::lnpbp::rgb::Amount;

use rgb::fungible::{Invoice, IssueStructure, Outcoins};
use rgb::i9n::*;
use rgb::rgbd::ContractName;
use rgb::util::SealSpec;

trait CReturnType: Sized + 'static {
    fn from_opaque(other: &COpaqueStruct) -> Result<&mut Self, String> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let ty = hasher.finish();

        if other.ty != ty {
            return Err(String::from("Type mismatch"));
        }

        let boxed = unsafe { Box::from_raw(other.ptr.clone() as *mut Self) };
        Ok(Box::leak(boxed))
    }
}
impl CReturnType for Runtime {}
impl CReturnType for String {}
impl CReturnType for () {}

#[repr(C)]
pub struct COpaqueStruct {
    ptr: *const c_void,
    ty: u64,
}

impl COpaqueStruct {
    fn new<T: 'static>(other: T) -> Self {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        let ty = hasher.finish();

        COpaqueStruct {
            ptr: Box::into_raw(Box::new(other)) as *const c_void,
            ty,
        }
    }

    fn raw<T>(ptr: *const T) -> Self {
        COpaqueStruct {
            ptr: ptr as *const c_void,
            ty: 0,
        }
    }
}

#[repr(C)]
pub struct CErrorDetails {
    message: *const c_char,
}

fn string_to_ptr(other: String) -> *const c_char {
    let cstr = match CString::new(other) {
        Ok(cstr) => cstr,
        Err(_) => CString::new(String::from(
            "Error converting string: contains a null-char",
        ))
        .unwrap(),
    };

    cstr.into_raw()
}

fn ptr_to_string(ptr: *mut c_char) -> Result<String, String> {
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.into())
            .map_err(|e| format!("{:?}", e))
    }
}

#[repr(C)]
pub enum CResultValue {
    Ok,
    Err,
}

#[repr(C)]
pub struct CResult {
    result: CResultValue,
    inner: COpaqueStruct,
}

impl<T: 'static, E> From<Result<T, E>> for CResult
where
    E: std::fmt::Debug,
{
    fn from(other: Result<T, E>) -> Self {
        match other {
            Ok(d) => CResult {
                result: CResultValue::Ok,
                inner: COpaqueStruct::new(d),
            },
            Err(e) => CResult {
                result: CResultValue::Err,
                inner: COpaqueStruct::raw(string_to_ptr(format!("{:?}", e))),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct StartRgbArgs {
    #[serde(with = "serde_with::rust::display_fromstr")]
    network: bp::Network,
    #[serde(with = "serde_with::rust::display_fromstr")]
    stash_endpoint: SocketLocator,
    contract_endpoints: HashMap<ContractName, String>,
    threaded: bool,
    datadir: String,
}

fn _start_rgb(json: *mut c_char) -> Result<Runtime, String> {
    let config: StartRgbArgs =
        serde_json::from_str(ptr_to_string(json)?.as_str()).map_err(|e| format!("{:?}", e))?;
    info!("Config: {:?}", config);

    let config = Config {
        network: config.network,
        stash_endpoint: config.stash_endpoint,
        threaded: config.threaded,
        data_dir: config.datadir,
        contract_endpoints: config
            .contract_endpoints
            .into_iter()
            .map(|(k, v)| -> Result<_, UrlError> { Ok((k, v.parse()?)) })
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{:?}", e))?,
    };

    Runtime::init(config).map_err(|e| format!("{:?}", e))
}

#[cfg(target_os = "android")]
fn start_logger() {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Debug));
}

#[cfg(not(target_os = "android"))]
fn start_logger() {}

#[no_mangle]
pub extern "C" fn start_rgb(json: *mut c_char) -> CResult {
    start_logger();

    info!("Starting RGB...");

    _start_rgb(json).into()
}

#[derive(Debug, Deserialize)]
struct IssueArgs {
    #[serde(with = "serde_with::rust::display_fromstr")]
    network: bp::Network,
    ticker: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    issue_structure: IssueStructure,
    #[serde(default)]
    allocations: Vec<Outcoins>,
    precision: u8,
    #[serde(default)]
    prune_seals: Vec<SealSpec>,
    #[serde(default)]
    dust_limit: Option<Amount>,
}

fn _issue(runtime: &COpaqueStruct, json: *mut c_char) -> Result<(), String> {
    let runtime = Runtime::from_opaque(runtime)?;
    let data: IssueArgs =
        serde_json::from_str(ptr_to_string(json)?.as_str()).map_err(|e| format!("{:?}", e))?;
    info!("{:?}", data);

    runtime
        .issue(
            data.network,
            data.ticker,
            data.name,
            data.description,
            data.issue_structure,
            data.allocations,
            data.precision,
            data.prune_seals,
            data.dust_limit,
        )
        .map_err(|e| format!("{:?}", e))
}

#[no_mangle]
pub extern "C" fn issue(runtime: &COpaqueStruct, json: *mut c_char) -> CResult {
    _issue(runtime, json).into()
}

#[derive(Debug, Deserialize)]
struct TransferArgs {
    inputs: Vec<OutPoint>,
    allocate: Vec<Outcoins>,
    #[serde(with = "serde_with::rust::display_fromstr")]
    invoice: Invoice,
    prototype_psbt: String,
    fee: u64,
    change: OutPoint,
    consignment_file: String,
    transaction_file: String,
}

fn _transfer(runtime: &COpaqueStruct, json: *mut c_char) -> Result<(), String> {
    let runtime = Runtime::from_opaque(runtime)?;
    let data: TransferArgs =
        serde_json::from_str(ptr_to_string(json)?.as_str()).map_err(|e| format!("{:?}", e))?;
    info!("{:?}", data);

    runtime
        .transfer(
            data.inputs,
            data.allocate,
            data.invoice,
            data.prototype_psbt,
            data.fee,
            data.change,
            data.consignment_file,
            data.transaction_file,
        )
        .map_err(|e| format!("{:?}", e))
        .map(|_| ())
    //.and_then(|r| serde_json::to_string(&r).map_err(|e| format!("{:?}", e)))
}

#[no_mangle]
pub extern "C" fn transfer(runtime: &COpaqueStruct, json: *mut c_char) -> CResult {
    _transfer(runtime, json).into()
}
