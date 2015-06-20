class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/archive/v1.2.0.tar.gz"
  sha256 "41675022a46801cc30bf3d31b546daf85e88054116f5d4a56f541c564523a0de"

  depends_on "rust"

  def install
    system *%w(cargo build --release)
    bin.install "target/release/hs"
  end

  test do
    system *%W(#{bin}/hs -v)
    assert_equal "cc\n", `echo 'aa\\nbb\\ncc\\n' | #{bin}/hs -s c -f`
  end
end
