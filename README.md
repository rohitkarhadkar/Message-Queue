# Message Queue Proof of Concept

## Purpose

This project serves to understand and implement messaging queues effectively. It was created as a Proof of Concept (POC) for the tech team at Pago, providing a foundational implementation that can be integrated into their project.

## Project Overview

This repository contains an implementation of a message queue system using PGMQ within a PostgreSQL instance. The core functionality allows an admin to send review requests to users, with a specified timeframe for the user to respond. If the user fails to provide a review within the allocated time, the request is moved to an `invalid_review_queue`. This system is designed for flexibility, allowing the review period to be set for testing or production use.

## Technologies Used

- **Rust**: Backend code
- **Actix Web**: Web framework for Rust
- **Tokio Postgres**: Asynchronous PostgreSQL client for Rust
- **PGMQ**: Message queue implemented within PostgreSQL
- **Docker**: Containerization of the PostgreSQL instance

## Setup Instructions

### Docker Container for PostgreSQL with PGMQ

1. **Run the Docker Container**:
    ```sh
    docker run -d --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 quay.io/tembo/pg16-pgmq:latest
    ```

2. **Connect to PostgreSQL**:
    ```sh
    psql postgres://postgres:postgres@0.0.0.0:5432/postgres
    ```

3. **Create the Queues**:
    ```sql
    SELECT pgmq.create('review_queue');
    SELECT pgmq.create('invalid_review_queue');
    ```

### Running the Backend Code

1. **Clone the Repository**:
    ```sh
    git clone <repository-url>
    cd <repository-directory>
    ```

2. **Set up the Rust Environment**:
    Ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

3. **Run the Backend**:
    ```sh
    cargo build
    cargo run
    ```

### API Endpoints

#### Push Review Request

- **Endpoint**: `POST /admin/push_review_request`
- **Body**:
    ```json
    {
        "user_id": 3,
        "message": "Please provide your review"
    }
    ```

#### Pop Review Request

- **Endpoint**: `POST /user/pop_review_request/{user_id}`

### Cron Job

The cron job runs every 10 seconds to check if any review requests have exceeded the 30-second window. If a review request is not fulfilled within 30 seconds, it is moved from `review_queue` to `invalid_review_queue`.


## Contributing

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes.
4. Commit your changes (`git commit -m 'Add new feature'`).
5. Push to the branch (`git push origin feature-branch`).
6. Create a new Pull Request.

## Contact

For any questions or feedback, please contact [Rohit Karhadkar](mailto:rohitkarhadkar.rk@gmail.com).
