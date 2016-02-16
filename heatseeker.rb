class Heatseeker < Formula
  desc "A high-performance general purpose fuzzy finder, based on selecta"
  homepage "https://github.com/rschmitt/heatseeker"
  url "https://github.com/rschmitt/heatseeker/archive/v1.4.0.tar.gz"
  sha256 "0988b722d8e20a58af74cadc53254b9493e1329faef78a7f33cae1ca7d92f7b3"

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
