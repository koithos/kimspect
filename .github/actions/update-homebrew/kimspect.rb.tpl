# Formula/kimspect.rb
class Kimspect < Formula
  desc "A CLI tool for kimspect"
  homepage "https://github.com/koithos/kimspect"
  version "${version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "${amd64_url}"
      sha256 "${amd64_sha256}"
    end
    if Hardware::CPU.arm?
      url "${arm64_url}"
      sha256 "${arm64_sha256}"
    end
  end

  def install
    if Hardware::CPU.intel?
      bin.install "kimspect-x86_64-apple-darwin" => "kimspect"
    elsif Hardware::CPU.arm?
      bin.install "kimspect-aarch64-apple-darwin" => "kimspect"
    end
  end

  test do
    system "#{bin}/kimspect", "--version"
  end
end
