# Clean room: borrowed behaviour, not borrowed artifacts

Specs, code, UI strings and docs carry no third-party chess product or format names; no proprietary
books, databases, art, or containers are bundled or referenced as data. Where behaviour was studied
from existing software (the reference database suite behind `port-reference-database-internals`),
algorithms and layouts are re-expressed as original code, and `NOTICE.md` records the provenance
commit-by-commit with `scripts/check-provenance.sh` enforcing it in CI: no upstream source line
enters this repository. Engine and protocol names users actually type (Stockfish, UCI, XBoard,
Nalimov/Syzygy) stay, because hiding the words users need would make the product unusable. The
rejected alternative — naming the reference products throughout the specs — would put the trademark
risk inside the public documentation instead of in one legal record.
