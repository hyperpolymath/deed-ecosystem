; SPDX-License-Identifier: MPL-2.0
;; guix.scm — GNU Guix package definition for a2ml_ex
;; Usage: guix shell -f guix.scm

(use-modules (guix packages)
             (guix build-system gnu)
             (guix licenses))

(package
  (name "a2ml_ex")
  (version "0.1.0")
  (source #f)
  (build-system gnu-build-system)
  (synopsis "a2ml_ex")
  (description "a2ml_ex — part of the hyperpolymath ecosystem.")
  (home-page "https://github.com/hyperpolymath/deed-ecosystem/tree/main/ex")
  (license ((@@ (guix licenses) license) "MPL-2.0"
             "https://github.com/hyperpolymath/palimpsest-license")))
