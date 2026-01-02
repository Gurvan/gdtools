extends Node

# gdlint:ignore=signal-name
signal BadSignal

signal AnotherBadSignal

# gdlint:disable=constant-name
const badConstant = 50
const anotherBad = 100
# gdlint:enable=constant-name

const thisWillWarn = 200
