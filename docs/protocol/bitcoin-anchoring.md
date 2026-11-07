# Bitcoin Checkpoint Anchoring

Bitcoin anchoring is an optional evidence layer, not APEX consensus, a bridge, or inherited Bitcoin security. A versioned checkpoint may commit to protocol version, APEX chain ID, height, block hash, state root, validator-set hash and previous checkpoint hash.

Track submitted, included and sufficiently confirmed states separately. Reorganizations can invalidate inclusion and require re-evaluation. Define fee bumping, congestion handling and confirmation policy. APEX must continue safely when Bitcoin is unavailable. An anchor proves neither correct execution nor honest APEX validators.
