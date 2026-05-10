# Homebrew Tap Setup

1. Build release binaries for each target:

   ```sh
   cargo build --release
   ```

2. Package the `tokenburn` binary into target-specific tarballs and upload them
   to the matching GitHub release.

3. Replace the placeholder `url` and `sha256` values in `tokenburn.rb`.

4. Publish the formula in a tap repository:

   ```sh
   brew tap your-org/tokenburn
   brew install tokenburn
   ```
