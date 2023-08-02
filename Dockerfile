FROM scratch
COPY target/x86_64-unknown-linux-gnu/release/medical_back /medical_back

ENTRYPOINT ["/medical_back"]
