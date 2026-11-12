# Fixtures

Everything here is **hand-written**, not recorded: the sandbox cannot reach any real
site. The pages imitate the *markup habits* of publisher and bookseller product pages
(JSON-LD shapes, Open Graph, microdata, CMS defects); the publishers, the bookseller,
the books, the authors and the ISBNs (valid check digits, invented numbers) are
fictional — not affiliated with any real company. No real cover, logo or page layout
is reproduced. Hosts use the reserved `.example` domain.

- `pages/` — HTML inputs for `scrape::extract`.
- `robots/` — robots.txt inputs for `robots::Robots`.

Binary image headers are generated inside the tests (dev-dependencies `image` and
`jpeg-encoder`, plus hand-assembled WebP and ISO-BMFF headers); none is committed.
