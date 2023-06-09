use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use anyhow::Error;
#[cfg(feature = "server")]
use http::request::Parts;
#[cfg(feature = "server")]
use http::{Method, Response};
#[cfg(feature = "server")]
use hyper::Body;
use percent_encoding::percent_decode_str;
use serde_json::Value;

use proxmox_schema::{ObjectSchema, ParameterSchema, ReturnType, Schema};

use super::Permission;
use crate::RpcEnvironment;
use crate::SerializableReturn;

/// A synchronous API handler gets a json Value as input and returns a json Value as output.
///
/// Most API handler are synchronous. Use this to define such handler:
/// ```
/// # use anyhow::Error;
/// # use serde_json::{json, Value};
/// use proxmox_router::{ApiHandler, ApiMethod, RpcEnvironment};
/// use proxmox_schema::ObjectSchema;
///
/// fn hello(
///    param: Value,
///    info: &ApiMethod,
///    rpcenv: &mut dyn RpcEnvironment,
/// ) -> Result<Value, Error> {
///    Ok(json!("Hello world!"))
/// }
///
/// const API_METHOD_HELLO: ApiMethod = ApiMethod::new(
///    &ApiHandler::Sync(&hello),
///    &ObjectSchema::new("Hello World Example", &[])
/// );
/// ```
pub type ApiHandlerFn = &'static (dyn Fn(Value, &ApiMethod, &mut dyn RpcEnvironment) -> Result<Value, Error>
              + Send
              + Sync
              + 'static);

/// A synchronous API handler gets a json Value as input and returns a serializable return value as output.
///
/// ```
/// # use anyhow::Error;
/// # use serde_json::{json, Value};
/// use proxmox_router::{ApiHandler, ApiMethod, RpcEnvironment, SerializableReturn};
/// use proxmox_schema::ObjectSchema;
///
/// fn hello(
///    param: Value,
///    info: &ApiMethod,
///    rpcenv: &mut dyn RpcEnvironment,
/// ) -> Result<Box<dyn SerializableReturn + Send>, Error> {
///    let res: Box<dyn SerializableReturn + Send> = Box::new(format!("Hello World!"));
///    Ok(res)
/// }
///
/// const API_METHOD_HELLO: ApiMethod = ApiMethod::new(
///    &ApiHandler::StreamingSync(&hello),
///    &ObjectSchema::new("Hello World Example", &[])
/// );
/// ```
pub type StreamingApiHandlerFn = &'static (dyn Fn(
    Value,
    &ApiMethod,
    &mut dyn RpcEnvironment,
) -> Result<Box<dyn SerializableReturn + Send>, Error>
              + Send
              + Sync
              + 'static);

/// Asynchronous API handlers
///
/// Returns a future Value.
/// ```
/// # use serde_json::{json, Value};
/// #
/// use proxmox_router::{ApiFuture, ApiHandler, ApiMethod, RpcEnvironment};
/// use proxmox_schema::ObjectSchema;
///
///
/// fn hello_future<'a>(
///    param: Value,
///    info: &ApiMethod,
///    rpcenv: &'a mut dyn RpcEnvironment,
/// ) -> ApiFuture<'a> {
///    Box::pin(async move {
///        let data = json!("hello world!");
///        Ok(data)
///    })
/// }
///
/// const API_METHOD_HELLO_FUTURE: ApiMethod = ApiMethod::new(
///    &ApiHandler::Async(&hello_future),
///    &ObjectSchema::new("Hello World Example (async)", &[])
/// );
/// ```
pub type ApiAsyncHandlerFn = &'static (dyn for<'a> Fn(Value, &'static ApiMethod, &'a mut dyn RpcEnvironment) -> ApiFuture<'a>
              + Send
              + Sync);

pub type ApiFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, anyhow::Error>> + Send + 'a>>;

/// Streaming asynchronous API handlers
///
/// Returns a future Value.
/// ```
/// # use serde_json::{json, Value};
/// #
/// use proxmox_router::{ApiFuture, ApiHandler, ApiMethod, RpcEnvironment, StreamingApiFuture, SerializableReturn};
/// use proxmox_schema::ObjectSchema;
///
///
/// fn hello_future<'a>(
///    param: Value,
///    info: &ApiMethod,
///    rpcenv: &'a mut dyn RpcEnvironment,
/// ) -> StreamingApiFuture<'a> {
///    Box::pin(async move {
///        let res: Box<dyn SerializableReturn + Send> = Box::new(format!("Hello World!"));
///        Ok(res)
///    })
/// }
///
/// const API_METHOD_HELLO_FUTURE: ApiMethod = ApiMethod::new(
///    &ApiHandler::StreamingAsync(&hello_future),
///    &ObjectSchema::new("Hello World Example (async)", &[])
/// );
/// ```
pub type StreamingApiAsyncHandlerFn = &'static (dyn for<'a> Fn(
    Value,
    &'static ApiMethod,
    &'a mut dyn RpcEnvironment,
) -> StreamingApiFuture<'a>
              + Send
              + Sync);

pub type StreamingApiFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Box<dyn SerializableReturn + Send>, anyhow::Error>> + Send + 'a>,
>;

/// Asynchronous HTTP API handlers
///
/// They get low level access to request and response data. Use this
/// to implement custom upload/download functions.
/// ```
/// # use serde_json::{json, Value};
/// #
/// use hyper::{Body, Response, http::request::Parts};
///
/// use proxmox_router::{ApiHandler, ApiMethod, ApiResponseFuture, RpcEnvironment};
/// use proxmox_schema::ObjectSchema;
///
/// fn low_level_hello(
///    parts: Parts,
///    req_body: Body,
///    param: Value,
///    info: &ApiMethod,
///    rpcenv: Box<dyn RpcEnvironment>,
/// ) -> ApiResponseFuture {
///    Box::pin(async move {
///        let response = http::Response::builder()
///            .status(200)
///            .body(Body::from("Hello world!"))?;
///        Ok(response)
///    })
/// }
///
/// const API_METHOD_LOW_LEVEL_HELLO: ApiMethod = ApiMethod::new(
///    &ApiHandler::AsyncHttp(&low_level_hello),
///    &ObjectSchema::new("Hello World Example (low level)", &[])
/// );
/// ```
#[cfg(feature = "server")]
pub type ApiAsyncHttpHandlerFn = &'static (dyn Fn(
    Parts,
    Body,
    Value,
    &'static ApiMethod,
    Box<dyn RpcEnvironment>,
) -> ApiResponseFuture
              + Send
              + Sync
              + 'static);

/// The output of an asynchronous API handler is a future yielding a `Response`.
#[cfg(feature = "server")]
pub type ApiResponseFuture =
    Pin<Box<dyn Future<Output = Result<Response<Body>, anyhow::Error>> + Send>>;

/// Enum for different types of API handler functions.
#[non_exhaustive]
pub enum ApiHandler {
    Sync(ApiHandlerFn),
    StreamingSync(StreamingApiHandlerFn),
    Async(ApiAsyncHandlerFn),
    StreamingAsync(StreamingApiAsyncHandlerFn),
    #[cfg(feature = "server")]
    AsyncHttp(ApiAsyncHttpHandlerFn),
}

#[cfg(feature = "test-harness")]
impl Eq for ApiHandler {}

#[cfg(feature = "test-harness")]
impl PartialEq for ApiHandler {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe {
            match (self, rhs) {
                (ApiHandler::Sync(l), ApiHandler::Sync(r)) => {
                    core::mem::transmute::<_, usize>(l) == core::mem::transmute::<_, usize>(r)
                }
                (ApiHandler::StreamingSync(l), ApiHandler::StreamingSync(r)) => {
                    core::mem::transmute::<_, usize>(l) == core::mem::transmute::<_, usize>(r)
                }
                (ApiHandler::Async(l), ApiHandler::Async(r)) => {
                    core::mem::transmute::<_, usize>(l) == core::mem::transmute::<_, usize>(r)
                }
                (ApiHandler::StreamingAsync(l), ApiHandler::StreamingAsync(r)) => {
                    core::mem::transmute::<_, usize>(l) == core::mem::transmute::<_, usize>(r)
                }
                #[cfg(feature = "server")]
                (ApiHandler::AsyncHttp(l), ApiHandler::AsyncHttp(r)) => {
                    core::mem::transmute::<_, usize>(l) == core::mem::transmute::<_, usize>(r)
                }
                _ => false,
            }
        }
    }
}

/// Lookup table to child `Router`s
///
/// Stores a sorted list of `(name, router)` tuples:
///
/// - `name`: The name of the subdir
/// - `router`: The router for this subdir
///
/// **Note:** The list has to be sorted by name, because we use a binary
/// search to find items.
///
/// This is a workaround unless RUST can const_fn `Hash::new()`
pub type SubdirMap = &'static [(&'static str, &'static Router)];

/// Classify different types of routers
pub enum SubRoute {
    //Hash(HashMap<String, Router>),
    /// Router with static lookup map.
    ///
    /// The first path element is used to lookup a new
    /// router with `SubdirMap`. If found, the remaining path is
    /// passed to that router.
    Map(SubdirMap),
    /// Router that always match the first path element
    ///
    /// The matched path element is stored as parameter
    /// `param_name`. The remaining path is matched using the `router`.
    MatchAll {
        router: &'static Router,
        param_name: &'static str,
    },
}

/// Macro to create an ApiMethod to list entries from SubdirMap
#[macro_export]
macro_rules! list_subdirs_api_method {
    ($map:expr) => {
        $crate::ApiMethod::new(
            &$crate::ApiHandler::Sync( & |_, _, _| {
                let index = ::serde_json::json!(
                    $map.iter().map(|s| ::serde_json::json!({ "subdir": s.0}))
                        .collect::<Vec<::serde_json::Value>>()
                );
                Ok(index)
            }),
            &$crate::ListSubdirsObjectSchema::new("Directory index.", &[])
                .additional_properties(true)
        ).access(None, &$crate::Permission::Anybody)
    }
}

/// Define APIs with routing information
///
/// REST APIs use hierarchical paths to identify resources. A path
/// consists of zero or more components, separated by `/`. A `Router`
/// is a simple data structure to define such APIs. Each `Router` is
/// responsible for a specific path, and may define `ApiMethod`s for
/// different HTTP requests (GET, PUT, POST, DELETE). If the path
/// contains more elements, `subroute` is used to find the correct
/// endpoint.
///
/// Routers are meant to be build a compile time, and you can use
/// all `const fn(mut self, ..)` methods to configure them.
///
///```
/// # use serde_json::{json, Value};
/// use proxmox_router::{ApiHandler, ApiMethod, Router};
/// use proxmox_schema::ObjectSchema;
///
/// const API_METHOD_HELLO: ApiMethod = ApiMethod::new(
///    &ApiHandler::Sync(&|_, _, _| {
///         Ok(json!("Hello world!"))
///    }),
///    &ObjectSchema::new("Hello World Example", &[])
/// );
/// const ROUTER: Router = Router::new()
///    .get(&API_METHOD_HELLO);
///```
pub struct Router {
    /// GET requests
    pub get: Option<&'static ApiMethod>,
    /// PUT requests
    pub put: Option<&'static ApiMethod>,
    /// POST requests
    pub post: Option<&'static ApiMethod>,
    /// DELETE requests
    pub delete: Option<&'static ApiMethod>,
    /// Used to find the correct API endpoint.
    pub subroute: Option<SubRoute>,
}

impl Router {
    /// Create a new Router.
    pub const fn new() -> Self {
        Self {
            get: None,
            put: None,
            post: None,
            delete: None,
            subroute: None,
        }
    }

    /// Configure a static map as `subroute`.
    pub const fn subdirs(mut self, map: SubdirMap) -> Self {
        self.subroute = Some(SubRoute::Map(map));
        self
    }

    /// Configure a `SubRoute::MatchAll` as `subroute`.
    pub const fn match_all(mut self, param_name: &'static str, router: &'static Router) -> Self {
        self.subroute = Some(SubRoute::MatchAll { router, param_name });
        self
    }

    /// Configure the GET method.
    pub const fn get(mut self, m: &'static ApiMethod) -> Self {
        self.get = Some(m);
        self
    }

    /// Configure the PUT method.
    pub const fn put(mut self, m: &'static ApiMethod) -> Self {
        self.put = Some(m);
        self
    }

    /// Configure the POST method.
    pub const fn post(mut self, m: &'static ApiMethod) -> Self {
        self.post = Some(m);
        self
    }

    /// Same as `post`, but expects an `AsyncHttp` handler.
    pub const fn upload(mut self, m: &'static ApiMethod) -> Self {
        // fixme: expect AsyncHttp
        self.post = Some(m);
        self
    }

    /// Same as `get`, but expects an `AsyncHttp` handler.
    pub const fn download(mut self, m: &'static ApiMethod) -> Self {
        // fixme: expect AsyncHttp
        self.get = Some(m);
        self
    }

    /// Same as `get`, but expects an `AsyncHttp` handler.
    pub const fn upgrade(mut self, m: &'static ApiMethod) -> Self {
        // fixme: expect AsyncHttp
        self.get = Some(m);
        self
    }

    /// Configure the DELETE method
    pub const fn delete(mut self, m: &'static ApiMethod) -> Self {
        self.delete = Some(m);
        self
    }

    /// Find the router for a specific path.
    ///
    /// - `components`: Path, split into individual components.
    /// - `uri_param`: Mutable hash map to store parameter from `MatchAll` router.
    pub fn find_route(
        &self,
        components: &[&str],
        uri_param: &mut HashMap<String, String>,
    ) -> Option<&Router> {
        if components.is_empty() {
            return Some(self);
        };

        let (dir, remaining) = (components[0], &components[1..]);

        let dir = match percent_decode_str(dir).decode_utf8() {
            Ok(dir) => dir.to_string(),
            Err(_) => return None,
        };

        match self.subroute {
            None => {}
            Some(SubRoute::Map(dirmap)) => {
                if let Ok(ind) = dirmap.binary_search_by_key(&dir.as_str(), |(name, _)| name) {
                    let (_name, router) = dirmap[ind];
                    //println!("FOUND SUBDIR {}", dir);
                    return router.find_route(remaining, uri_param);
                }
            }
            Some(SubRoute::MatchAll { router, param_name }) => {
                //println!("URI PARAM {} = {}", param_name, dir); // fixme: store somewhere
                uri_param.insert(param_name.to_owned(), dir);
                return router.find_route(remaining, uri_param);
            }
        }

        None
    }

    /// Lookup the API method for a specific path.
    /// - `components`: Path, split into individual components.
    /// - `method`: The HTTP method.
    /// - `uri_param`: Mutable hash map to store parameter from `MatchAll` router.
    #[cfg(feature = "server")]
    pub fn find_method(
        &self,
        components: &[&str],
        method: Method,
        uri_param: &mut HashMap<String, String>,
    ) -> Option<&ApiMethod> {
        if let Some(info) = self.find_route(components, uri_param) {
            return match method {
                Method::GET => info.get,
                Method::PUT => info.put,
                Method::POST => info.post,
                Method::DELETE => info.delete,
                _ => None,
            };
        }
        None
    }
}

impl Default for Router {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

const NULL_SCHEMA: Schema = Schema::Null;

fn dummy_handler_fn(
    _arg: Value,
    _method: &ApiMethod,
    _env: &mut dyn RpcEnvironment,
) -> Result<Value, Error> {
    // do nothing
    Ok(Value::Null)
}

const DUMMY_HANDLER: ApiHandler = ApiHandler::Sync(&dummy_handler_fn);

/// Access permission with description
#[cfg_attr(feature = "test-harness", derive(Eq, PartialEq))]
pub struct ApiAccess {
    pub description: Option<&'static str>,
    pub permission: &'static Permission,
}

/// This struct defines a synchronous API call which returns the result as json `Value`
#[cfg_attr(feature = "test-harness", derive(Eq, PartialEq))]
pub struct ApiMethod {
    /// The protected flag indicates that the provides function should be forwarded
    /// to the daemon running in privileged mode.
    pub protected: bool,
    /// This flag indicates that the provided method may change the local timezone, so the server
    /// should do a tzset afterwards
    pub reload_timezone: bool,
    /// Parameter type Schema
    pub parameters: ParameterSchema,
    /// Return type Schema
    pub returns: ReturnType,
    /// Handler function
    pub handler: &'static ApiHandler,
    /// Access Permissions
    pub access: ApiAccess,
}

impl std::fmt::Debug for ApiMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ApiMethod {{ ")?;
        write!(f, "  parameters: {:?}", self.parameters)?;
        write!(f, "  returns: {:?}", self.returns)?;
        write!(f, "  handler: {:p}", &self.handler)?;
        write!(f, "  permissions: {:?}", &self.access.permission)?;
        write!(f, "}}")
    }
}

impl ApiMethod {
    pub const fn new_full(handler: &'static ApiHandler, parameters: ParameterSchema) -> Self {
        Self {
            parameters,
            handler,
            returns: ReturnType::new(false, &NULL_SCHEMA),
            protected: false,
            reload_timezone: false,
            access: ApiAccess {
                description: None,
                permission: &Permission::Superuser,
            },
        }
    }

    pub const fn new(handler: &'static ApiHandler, parameters: &'static ObjectSchema) -> Self {
        Self::new_full(handler, ParameterSchema::Object(parameters))
    }

    pub const fn new_dummy(parameters: &'static ObjectSchema) -> Self {
        Self {
            parameters: ParameterSchema::Object(parameters),
            handler: &DUMMY_HANDLER,
            returns: ReturnType::new(false, &NULL_SCHEMA),
            protected: false,
            reload_timezone: false,
            access: ApiAccess {
                description: None,
                permission: &Permission::Superuser,
            },
        }
    }

    pub const fn returns(mut self, returns: ReturnType) -> Self {
        self.returns = returns;

        self
    }

    pub const fn protected(mut self, protected: bool) -> Self {
        self.protected = protected;

        self
    }

    pub const fn reload_timezone(mut self, reload_timezone: bool) -> Self {
        self.reload_timezone = reload_timezone;

        self
    }

    pub const fn access(
        mut self,
        description: Option<&'static str>,
        permission: &'static Permission,
    ) -> Self {
        self.access = ApiAccess {
            description,
            permission,
        };

        self
    }
}
