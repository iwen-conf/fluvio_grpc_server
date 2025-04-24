fn main() {

    tonic_build::compile_protos("./proto/fluvio_grpc.proto").unwrap();

}