class Tokenburn < Formula
  desc "Track token usage and burn for Claude Code and Codex"
  homepage "https://github.com/your-org/tokenburn"
  version "1.0.0"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/your-org/tokenburn/releases/download/v1.0.0/tokenburn-aarch64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_ARM64_SHA256"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/your-org/tokenburn/releases/download/v1.0.0/tokenburn-x86_64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_X86_64_SHA256"
  else
    odie "tokenburn currently ships prebuilt macOS binaries only"
  end

  def install
    bin.install "tokenburn"
  end

  test do
    assert_match "tokenburn", shell_output("#{bin}/tokenburn --help")
  end
end
