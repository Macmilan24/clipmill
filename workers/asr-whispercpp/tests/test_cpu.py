import pytest
from clipmill_worker_asr_whispercpp import engine
from clipmill_worker_asr_whispercpp.cpu import missing_instructions
from clipmill_worker_sdk import DeterministicTaskError

IVY_BRIDGE = {"AVX": True, "AVX2": False, "FMA3": False, "F16C": True}
HASWELL = {"AVX": True, "AVX2": True, "FMA3": True, "F16C": True, "BMI2": True}


def test_an_avx_only_processor_is_named_for_what_it_lacks():
    assert missing_instructions(IVY_BRIDGE, "AMD64", dispatched=False) == ["AVX2", "FMA3", "BMI2"]
    assert missing_instructions(IVY_BRIDGE, "AMD64", dispatched=True) == []


def test_a_haswell_or_newer_processor_lacks_nothing():
    assert missing_instructions(HASWELL, "x86_64") == []


def test_a_vm_without_bmi2_cannot_import_the_stock_wheel():
    filtered = HASWELL | {"BMI2": False}
    assert missing_instructions(filtered, "AMD64", dispatched=False) == ["BMI2"]
    assert missing_instructions(filtered, "AMD64", dispatched=True) == []


def test_other_architectures_are_not_asked():
    assert missing_instructions({}, "arm64") == []


def test_this_machine_answers_without_error():
    assert isinstance(missing_instructions(), list)


def test_stock_incompatible_binding_is_refused_before_its_native_import(monkeypatch):
    monkeypatch.setattr(engine, "missing_instructions", lambda: ["AVX2"])
    with pytest.raises(DeterministicTaskError, match="update Transcription"):
        engine._runtime()
