FROM rust:slim AS compiler

    # BUILD-TIME
        # setup workdir
        WORKDIR /app
        RUN cargo new compile

        WORKDIR /app/compile/src
        RUN rm -f main.rs
        RUN mkdir -p bin

        # copy Cargo.toml + Cargo.lock (necessary for dependecies)
        WORKDIR /app/compile
        COPY Cargo.toml .
        COPY Cargo.lock .

    # RUN-TIME
        #execute a script when container start
        # 'sleep infinity' to keep-alive cmd (container need to just run)
        CMD ["sh", "-c", "rm -f src/bin/{*,.*}; rm -rf target; sleep infinity"]

############################
# cargo build --release --bin proj_123

### cmd to build
# docker build -t rust_compiler .

### cmd to create volume
# docker volume create shared_folder

### cmd to run
# docker run -it --name=ruscompy --mount source=shared_folder,target=/app/shared_folder rust_compiler
### OR cmd to create + then run
# docker create --name=ruscompy --mount source=shared_folder,target=/app/shared_folder rust_compiler
# docker start ruscompy


### COMPILER structure
# app <- base
# app/shared_folder <- volume 'shared_folder' mounted here
# app/compile <- contains Cargo.toml + Cargo.lock + src 
# app/compile/src/bin <- contains all temp <file_name>.rs of clients
# app/compile/target/release <- contains all temp <file_name> executable of clients