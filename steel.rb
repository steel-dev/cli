# This needs a github action to be updated
class Steel < Formula
  desc "Steel CLI"
  homepage "https://github.com/steel-dev/cli"
  url "https://registry.npmjs.org/@steel-dev/cli/-/cli-0.0.4.tgz"
  sha256 "8b41fc4832eb381dd67d563262f4feba893dfa9ad5cee13aab20887a9c14d08a"
  license "MIT"

  depends_on "node"

  def install
    system "npm", "install", *Language::Node.std_npm_install_args(libexec), "@steel-dev/cli"
    bin.install_symlink Dir["#{libexec}/bin/*"]
  end

  test do
    system "#{bin}/steel", "--help"
  end
end
