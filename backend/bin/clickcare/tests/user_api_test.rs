#[cfg(test)]
mod test {
    use dotenvy::dotenv;
    use dtor::dtor;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::user_api_impl::UserApiImpl;
    use clickcare::infrastructure::grpc::user_api_server::UserApiServer;
    use clickcare::infrastructure::grpc::SignUpRequest;
    use rstest::*;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, ImageExt};
    use testcontainers_modules::postgres;
    use tokio::net::TcpListener;
    use tokio::sync::OnceCell;
    use tonic::transport::Server;
    use clickcare::infrastructure::log::init_logger;

    struct TestEnv {
        grpc_addr: String,
        // Mantiene el contenedor vivo durante toda la suite.
        _container: ContainerAsync<postgres::Postgres>,
    }

    static TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

    async fn test_env() -> &'static TestEnv {
        TEST_ENV
            .get_or_init(|| async {
                init_logger();
                dotenv().ok();

                let user = "admin";
                let password = "admin123";
                let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

                let container = postgres::Postgres::default()
                    .with_user(user)
                    .with_password(password)
                    .with_container_name(format!("clickcare-test-{}", timestamp))
                    .start()
                    .await
                    .unwrap();

                let host_port = container.get_host_port_ipv4(5432).await.unwrap();
                let connection_string = format!(
                    "postgres://{}:{}@127.0.0.1:{host_port}/postgres",
                    user, password
                );

                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();

                let service = UserApiImpl::new(Some(connection_string)).await.unwrap();

                tokio::spawn(async move {
                    Server::builder()
                        .add_service(UserApiServer::new(service))
                        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                            listener,
                        ))
                        .await
                        .unwrap();
                });

                TestEnv {
                    grpc_addr: format!("http://{}", addr),
                    _container: container,
                }
            })
            .await
    }

    #[fixture]
    async fn grpc_server_addr() -> &'static str {
        test_env().await.grpc_addr.as_str()
    }

    #[dtor]
    fn shutdown() {
        if let Some(env) = TEST_ENV.get() {
            println!("Shutting down test environment with gRPC server at {}", env.grpc_addr);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Aquí podrías agregar lógica para limpiar la base de datos o realizar otras tareas de limpieza si es necesario.
                let _ = env._container.stop().await;
            });
        }
    }

    mod uuid_test {
        use log::info;
        use rstest::rstest;
        use clickcare::infrastructure::grpc::SignUpRequest;
        use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
        use crate::test::grpc_server_addr;

        #[rstest]
        #[tokio::test]
        async fn test_user_id_error_uuid_v4(#[future(awt)] grpc_server_addr: &'static str) {
            let mut client = UserApiClient::connect(grpc_server_addr)
                .await
                .expect("Fallo al conectar");

            let request = tonic::Request::new(SignUpRequest {
                id_token: "test-token".into(),
                user_id: uuid::Uuid::new_v4().to_string(),
                email: "test@example.com".into(),
                ..Default::default()
            });

            let response = client.sign_up(request).await;

            info!("Response: {:?}", response);
            assert!(response.is_err());
            let _inner = response.unwrap().into_inner();
            // Validar contenido del response
        }

        #[rstest]
        #[tokio::test]
        async fn test_sign_up_grpc_integration(#[future(awt)] grpc_server_addr: &'static str) {
            let mut client = UserApiClient::connect(grpc_server_addr)
                .await
                .expect("Fallo al conectar");

            let request = tonic::Request::new(SignUpRequest {
                id_token: "test-token".into(),
                user_id: uuid::Uuid::now_v7().to_string(),
                email: "test@example.com".into(),
                ..Default::default()
            });

            let response = client.sign_up(request).await;

            info!("Response: {:?}", response);
            assert!(response.is_ok());
            let _inner = response.unwrap().into_inner();
            // Validar contenido del response
        }
    }

}
