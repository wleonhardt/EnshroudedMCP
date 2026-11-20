"""Record one authorized observation before caller interpretation; no retry logic."""


class ObservationFailed(RuntimeError):
    """A bounded transport/parse/recording reason without raw exception text."""


def _record(record, event):
    try:
        record(event)
    except OSError:
        raise ObservationFailed("observation_record_unconfirmed") from None


def observe(label: str, action, record):
    """Bind action to one authorized observation and record to a synchronous journal.

    The caller supplies fixed scope/identity and a recorder that flushes/fsyncs
    before returning. This helper enforces ordering, not filesystem durability.
    Native MCP refusal replies are retained and returned unchanged for the caller
    to interpret. Raw replies belong in private evidence; arguments and exception
    text are not copied. An intent without a reply is incomplete observation,
    never execution, cancellation, exit, save or replay authority.
    """
    _record(record, {"kind": "observation_intent", "label": label})
    reason = None
    try:
        reply = action()
    except OSError:
        reason = "transport_error"
    except ValueError:
        reason = "invalid_reply"
    if reason is not None:
        _record(record, {"kind": "observation_error", "label": label, "reason": reason})
        raise ObservationFailed(reason) from None
    _record(record, {"kind": "observation_reply", "label": label, "reply": reply})
    return reply
