import sys
import os
from typing import List, Tuple

sys.path.append(os.path.join(os.path.dirname(__file__), ".."))

import platform
from pathlib import Path

from pytest_benchmark.fixture import BenchmarkFixture
from hypothesis import given, settings
import hypothesis.strategies as st

if platform.system() == "Linux":
    assert (Path(__file__).parent.parent / "src" / "my_library.so").is_file()
    # TODO On windows, check if correct filename exists
from src import my_library


@settings(max_examples=1000, deadline=2)
@given(n=st.integers(min_value=-(2 ** 127), max_value=2 ** 127))
def test_add_one(n: int):
    assert my_library.add_one(n) == n + 1


@settings(max_examples=1000, deadline=2)
@given(n=st.floats(allow_nan=False, allow_infinity=False, min_value=-(2 ** 127), max_value=2 ** 127))
def test_add_one_and_a_half(n: float):
    assert my_library.add_one_and_a_half(n) == n + 1.5


@settings(max_examples=1000, deadline=2)
@given(n=st.text())
def test_concatenate_string(n: str):
    assert my_library.concatenate_string(n) == n + " world!"


@settings(max_examples=1000, deadline=2)
@given(n=st.lists(st.integers(min_value=-(2 ** 100), max_value=2 ** 100)))
def test_sum_of_list(n: List[int]):
    python_sum = sum(n)
    if -(2 ** 127) < python_sum < 2 ** 127:
        assert my_library.sum_of_list(n) == python_sum


@settings(max_examples=1000, deadline=2)
@given(n=st.lists(st.integers(min_value=-(2 ** 100), max_value=2 ** 100)))
def test_append_to_list(n: List[int]):
    copy_n = n.copy()
    my_library.append_to_list(n)
    assert 420 in n
    assert len(copy_n) + 1 == len(n)
    assert n[-1] == 420


@settings(max_examples=1000, deadline=2)
@given(n=st.lists(st.integers(min_value=-(2 ** 126), max_value=2 ** 126)))
def test_double_of_list(n: List[int]):
    assert my_library.double_of_list(n) == [i * 2 for i in n]


@settings(max_examples=1000, deadline=2)
@given(
    st.tuples(
        st.integers(min_value=-(2 ** 126), max_value=2 ** 126), st.integers(min_value=-(2 ** 126), max_value=2 ** 126)
    )
)
def test_tuple_interaction(my_tuple: Tuple[int, int]):
    a, b = my_tuple
    assert my_library.tuple_interaction(my_tuple) == (a, b, a + b)


# TODO Dictionaries
# TODO Sets
# TODO Big number (factorial)
# TODO Numpy arrays
# TODO Pathfinding when implemented

# TODO Benchmarks
# Which numpy conversion to and from rust is the fastest?
# Which pathfinding algorithm is the fastest? Are there differences when the path distance is small/large?
# Which pathfinding algorithm to use when trying to find paths from multiple sources to one target (or one source to multiple targets)


def test_tuple_interaction_bench(benchmark: BenchmarkFixture):
    my_tuple = (1, 2)
    _result = benchmark(my_library.tuple_interaction, my_tuple)
