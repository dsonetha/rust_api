# Rest API
A basic API to manage users with GET/POST/DELETE routes and jwt auth.

# Setup
Make sure to have postgresql instance running and set up your variables in the `.env` file.
Install diesel cli if needed and run migrations.
```bash
cargo install diesel_cli --no-default-features --features postgres
diesel setup
diesel migration run
```

# Other, todo
Handle invalidate jwt tokens, for logout and DELETE user.
Update users.
Admin role and routes.
