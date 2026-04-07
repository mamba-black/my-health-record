#[cfg(test)]
mod test {
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::user_api_impl::UserApiImpl;
    use clickcare::infrastructure::grpc::user_api_server::UserApiServer;
    use clickcare::infrastructure::grpc::SignUpRequest;
    use rstest::*;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres;
    use tokio::net::TcpListener;
    use tonic::transport::Server;


    #[fixture]
    async fn grpc_server_addr() -> String {
        let user = "admin";
        let password = "admin123";
        let container = postgres::Postgres::default()
            .with_user(user)
            .with_password(password)
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!("postgres://{}:{}@127.0.0.1:{host_port}/postgres", user, password);

        // 1. Buscar un puerto libre
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 2. Instanciar el servicio (con mocks si prefieres)
        let service = UserApiImpl::new(Some(connection_string)).await.unwrap();

        // 3. Correr el servidor en el fondo
        tokio::spawn(async move {
            Server::builder()
                .add_service(UserApiServer::new(service))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .unwrap();
        });

        format!("http://{}", addr)
    }

    #[rstest]
    #[tokio::test]
    async fn test_sign_up_grpc_integration(#[future] grpc_server_addr: String) {
        let addr = grpc_server_addr.await;

        // 4. Crear el cliente
        let mut client = UserApiClient::connect(addr)
            .await
            .expect("Fallo al conectar");

        // 5. Ejecutar la llamada
        let request = tonic::Request::new(SignUpRequest {
            id_token: "test-token".into(),
            user_id: uuid::Uuid::now_v7().to_string(), // Usando UUID v7 como pide tu repo
            email: "test@example.com".into(),
            // ... el resto de campos
            ..Default::default()
        });

        let response = client.sign_up(request).await;

        // 6. Aserciones
        assert!(response.is_ok());
        let inner = response.unwrap().into_inner();
        // Validar contenido del response
    }
}
