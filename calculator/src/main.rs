use proto::admin_server::{Admin, AdminServer};
// Make sure to import the calculator trait and the server from the proto module
use proto::calculator_server::{Calculator, CalculatorServer};
use tonic::transport::Server;

/*
 * Allows namespacing of generated code from the protobuf files. 
 * 
 * The `tonic::include_proto!`macro is used to import the generated code for the `calculator` 
 * package defined in the protobuf files. Remember that you define the package name in the proto file
 */
mod proto {
    tonic::include_proto!("calculator");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("calculator_descriptor");
}

/* 
 * Type alias for the shared state used to track request counts.
 */
type State = std::sync::Arc<tokio::sync::RwLock<u64>>;

 /*
  * To create the actual service, we define a struct that implements the generated service traits.
  * In this case, we define a `CalculatorService` struct that implements the `Calculator` trait generated from the protobuf file. 
  * The `CalculatorService` struct has a field `state` which is an `Arc<RwLock<u64>>` to keep track of the number of requests made to the service. 
  * The `increment_counter` method is used to increment the request count in a thread-safe manner.
  */
#[derive(Debug, Default)]
struct CalculatorService {
    state: State,
}

/* 
 * The `increment_counter` method is an asynchronous function that increments the request count. 
 * It acquires a write lock on the `state` to safely update the count and prints the current count to the console.
 */
impl CalculatorService {
    async fn increment_counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
        println!("Request count: {}", *count);
    }
}

/*
 * The `Calculator` trait is implemented for the `CalculatorService` struct.
 * This trait is generated from the protobuf file and defines the gRPC methods that the service will implement. 
 * In this case, we implement the `add` and `divide` methods, which perform addition and division operations respectively. 
 * 
 * Because tonic uses tokio under the hood, the traits contain async methods, and we use the `#[tonic::async_trait]`
 * attribute to allow async trait methods.
 */
#[tonic::async_trait]
impl Calculator for CalculatorService {
    async fn add(
        &self,
        request: tonic::Request<proto::CalculationRequest>, // Called with a CalculationRequest message, which contains the two numbers
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> { // Returns a CalculationResponse message, which contains the result or a Status in case of an error
        self.increment_counter().await;

        /*  
         * The `get_ref` method is used to access the inner message of the request, which is a `CalculationRequest` 
         * containing the two numbers to which some function (add or divide) will be performed on
         */
        let input = request.get_ref();


        /*
         * This is where the actual logic of the service method is implemented. 
         * In this case, we perform the addition of the two numbers
         */
        let response = proto::CalculationResponse {
            result: input.a + input.b,
        };

        /*
         * Wrap the response in a `tonic::Response` which will then be wrapped in Ok(()) to indicate a successful response.
         */
        Ok(tonic::Response::new(response))
    }

    async fn divide(
        &self,
        request: tonic::Request<proto::CalculationRequest>, //
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        self.increment_counter().await;

        let input = request.get_ref();

        // We check for division by zero and return an appropriate error status if that is the case
        if input.b == 0 {
            return Err(tonic::Status::invalid_argument("Cannot divide by zero"));
        }

        let response = proto::CalculationResponse {
            result: input.a / input.b,
        };

        Ok(tonic::Response::new(response))
    }
}

/* 
 * We also define an `AdminService` struct that implements the `Admin` trait generated from the protobuf file. 
 * This service provides an endpoint to get the current request count, which is useful for monitoring and debugging purposes.
 */
#[derive(Default, Debug)]
struct AdminService {
    state: State,
}

/* 
 * The `Admin` trait is implemented for the `AdminService` struct. 
 * This trait is generated from the protobuf file and defines the gRPC methods that the admin service will implement. 
 * In this case, we implement the `get_request_count` method, which returns the current request count.
 */
#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _request: tonic::Request<proto::GetCountRequest>,
    )  -> Result<tonic::Response<proto::CounterResponse>, tonic::Status> {
        let count = self.state.read().await;
        let response = proto::CounterResponse { count: *count };

        Ok(tonic::Response::new(response))
    }
}

use tonic::metadata::MetadataValue;
use tonic::{Request, Status};

/* 
 * Function to check authentication for incoming requests
 */
fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    // In a real application, you would validate the token properly, possibly against a database or an authentication service.
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    // Check the "authorization" metadata in the request against the expected token
    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req), // If the token matches, allow the request to proceed
        _ => Err(Status::unauthenticated("No valid auth token")), // If the token is missing or invalid, return an unauthenticated status
    }
}

/* 
 * Main function to start the gRPC server
 * 
 * The `#[tokio::main]` attribute is used to indicate that this is the entry point of a Tokio async runtime.
 
 * The server is configured to listen on the address "[::1]:50051" and serves both the `CalculatorService` and `AdminService`.
 * 
 * The `tonic_reflection` service is also added to enable gRPC reflection, which allows clients to query the server for its available
 * services and methods at runtime.
 * 
 * CORS is enabled using `tower_http::cors::CorsLayer` to allow gRPC-Web clients to access the server without needing a proxy like Envoy.
 * 
 * The `check_auth` function is used as an interceptor for the `AdminService` to ensure that only authenticated requests can access the 
 * admin endpoints.
 * 
 * Finally, the server is started and will run until it is stopped or encounters an error.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?; // Listen on localhost at port 50051

    let state = State::default(); // Shared state for tracking request counts

    let calc = CalculatorService { // Create an instance of the CalculatorService with the shared state
        state: state.clone(),
    };

    let admin = AdminService { // Create an instance of the AdminService with the shared
        state: state.clone(),
    };

    let service = tonic_reflection::server::Builder::configure() // Configure the reflection service
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET) // Register the file descriptor set for reflection, which allows clients to query the server for its available services and methods at runtime
        .build()?; // Build the reflection service

    Server::builder()
        .accept_http1(true) // Enable HTTP/1.1 support to allow gRPC-Web clients to connect without needing a proxy like Envoy
        .layer(tower_http::cors::CorsLayer::permissive()) // Enable CORS with permissive settings to allow gRPC-Web clients to access the server
        .add_service(service) // Add the reflection service to the server
        .add_service(tonic_web::enable(CalculatorServer::new(calc))) // Add the CalculatorService to the server, wrapped with tonic_web::enable to allow gRPC-Web support
        .add_service(AdminServer::with_interceptor(admin, check_auth)) 
        .serve(addr) // Start the server and listen on the specified address
        .await?; // Await the server to run until it is stopped or encounters an error
    Ok(())
}
