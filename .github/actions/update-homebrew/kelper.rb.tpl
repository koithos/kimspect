# Formula/kelper.rb
class Kelper < Formula
  desc "A CLI tool for Kelper"
  homepage "https://github.com/aliabbasjaffri/kelper"
  version "{{version}}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "{{amd64_url}}"
      sha256 "{{amd64_sha256}}"
    end
    if Hardware::CPU.arm?
      url "{{arm64_url}}"
      sha256 "{{arm64_sha256}}"
    end
  end

  def install
    if Hardware::CPU.intel?
      bin.install "kelper-x86_64-apple-darwin" => "kelper"
    elsif Hardware::CPU.arm?
      bin.install "kelper-aarch64-apple-darwin" => "kelper"
    end
  end

  test do
    system "#{bin}/kelper", "--version"
  end
end
