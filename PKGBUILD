# PKGBUILD
pkgname=logmill-git
pkgver=0.1.0
pkgrel=1
pkgdesc="A zero-loss, memory-safe proxy log rotator built in Rust."
arch=('x86_64')
url="https://github.com/wrawler/logmill"
license=('MIT')
makedepends=('cargo')
provides=('logmill')
conflicts=('logmill')
source=("logmill::git+https://github.com/wrawler/logmill.git")
md5sums=('SKIP')

build() {
  cd "$srcdir/logmill"
  cargo build --release --locked
}

package() {
  cd "$srcdir/logmill"
  
  # Install the binary
  install -Dm755 "target/release/logmill" "$pkgdir/usr/bin/logmill"
  
  # Install the LICENSE
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/${pkgname}/LICENSE"
}