# Maintainer: Peter Henry <me@peterhenry.net>
pkgname="fluidspaces-rs-git"
_pkgname="fluidspaces"
pkgver="0.4.1.r5.gb27d3a3"
pkgrel=1
pkgdesc="Daemon written in Rust to manage dynamically named i3 workspaces."
arch=('x86_64')
url="https://github.com/mosbasik/fluidspaces-rs"
license=('Apache')
depends=('dmenu')
makedepends=("rustup" "cargo" "git")
# optdepends=()
provides=($_pkgname)
conflicts=($_pkgname)
# replaces=()
# backup=()
options=()
# install=
# changelog=
source=("$pkgname::git+https://github.com/mosbasik/fluidspaces-rs.git#branch=feature/arch-package")
# noextract=()
# md5sums=() #autofill using updpkgsums
sha256sums=('SKIP')

pkgver() {
  cd "$pkgname"
  # echo "$(grep '^version =' Cargo.toml|head -n1|cut -d\" -f2).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)"
  git describe --long --tags | sed 's/^v// ; s/\([^-]*-g\)/r\1/ ; s/-/\./g'
}

build () {
  cd "$pkgname"
  env CARGO_INCREMENTAL=0 cargo build --release
}

package() {
  cd "$pkgname"

  install -D -m755 "$srcdir/$pkgname/target/release/fluidspaces" "$pkgdir/usr/bin/fluidspaces"
  install -D -m755 "$srcdir/$pkgname/target/release/fluidspaces-msg" "$pkgdir/usr/bin/fluidspaces-msg"

  sed -i "s:ExecStart=/usr/bin/fluidspaces:ExecStart=${pkgdir}/usr/bin/fluidspaces:" "fluidspaces.service"
  install -D -m755 "$srcdir/$pkgname/fluidspaces.service" "$pkgdir/usr/lib/systemd/user/fluidspaces.service"
}
