cargo build --release 
# binary at ./target/release/reefmt
# optionally:
Copy-Item ./target/release/reefmt.exe .
# Remove build artifacts (binary was copied above)
Remove-Item ./target -Recurse -Force

# reefmt routes/authors/form.ree
# reefmt **/*.ree
# reefmt views/users/show.ree routes/home.ree

# ./target/release/reefmt routes/authors/form.ree
