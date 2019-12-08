import time
import numpy as np
import my_library


def my_factorial(n):
    if n == 1:
        return 1
    return n * my_factorial(n - 1)


input = 34
t1 = time.perf_counter()
output = my_library.factorial(input)
t2 = time.perf_counter()
output2 = my_factorial(input)
t3 = time.perf_counter()
print(f"{input}! = Rust: {output} ({type(output)}) = Py: {output2}")
print(f"Time passed: {t2-t1} and {t3-t2}")
assert output == output2, f"{output} != {output2}"


string_output = my_library.sum_as_string(7, 6)
assert string_output == "13"


p1 = my_library.Point2d.origin()
print(p1)
p2 = my_library.Point2d(3.0, 4.0)
print(p2)
dist = p1.distance_to(p2)
print(dist)
dist_squared = p1.distance_to_squared(p2)
print(dist_squared)


my_list = [0, 1, 2, 3, 4]
my_array = np.asarray([0, 1, 2, 3, 4])
# TODO: read numpy arrays in rust functions / structs
