FROM debian:bookworm-slim as runner

    # BUILD-TIME
        # setup workdir
        WORKDIR /app
        RUN mkdir -p run
        WORKDIR /app/run

    # RUN-TIME
        #execute a script when container start
        # 'sleep infinity' to keep-alive cmd (container need to just run)
        CMD ["sh", "-c", "rm -f {*,.*}; rm -f ../shared_folder/{*,.*}; sleep infinity"]

############################
### cmd to build
# docker build -t rust_runner .

### cmd to create volume
# docker volume create shared_folder

### cmd to run
# docker run -it --name=ruruny --mount source=shared_folder,target=/app/shared_folder rust_runner
### OR cmd to create + then start
# docker create --name=ruruny --mount source=shared_folder,target=/app/shared_folder rust_runner
# docekr start ruruny


### RUNNER structure
# app <- base
# app/shared_folder <- volume 'shared_folder' mounted here
# app/run <- contains all temp <file_name> executable of clients