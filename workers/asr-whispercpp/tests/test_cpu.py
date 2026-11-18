from clipmill_worker_asr_whispercpp.cpu import missing_instructions

IVY_BRIDGE = {"AVX": True, "AVX2": False, "FMA3": False, "F16C": True}
HASWELL = {"AVX": True, "AVX2": True, "FMA3": True, "F16C": True}


def test_an_avx_only_processor_is_named_for_what_it_lacks():
    assert missing_instructions(IVY_BRIDGE, "AMD64") == ["AVX2", "FMA3"]


def test_a_haswell_or_newer_processor_lacks_nothing():
    assert missing_instructions(HASWELL, "x86_64") == []


def test_other_architectures_are_not_asked():
    assert missing_instructions({}, "arm64") == []


def test_this_machine_answers_without_error():
    assert isinstance(missing_instructions(), list)
