import time
import numpy as np
import my_library


def my_factorial(n):
    if n == 1:
        return 1
    return n * my_factorial(n - 1)

class Point2:
    def __init__(self, x, y):
        self.x = x
        self.y = y

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


p1 = my_library.Point2d(0, 0)
print(p1)
p2 = my_library.Point2d(3.0, 4.0)
print(p2)
dist = p1.distance_to(p2)
print(dist)
dist_squared = p1.distance_to_squared(p2)
print(dist_squared)

ps = my_library.Point2Collection([p1, p2])
print(f"Amount of points in the list: {ps.len()}")
# Add a Python Point2 point to the list of points
p3 = Point2(7, 8)
ps.append(p3)
ps.append(p1)
print(f"Amount of points in the list: {ps.len()}")
# This does not do anything:
ps.points.append(p3)
print(f"Amount of points in the list: {ps.len()}")
# However you can set the list directly
# ps.points = [Point2(1, 2), Point2(2, 3), Point2(3, 4)]
# print(f"Amount of points in the list: {ps.len()}")
print()
print(f"The points in the list:")
ps.print()

p4 = my_library.Point2d(9, 10)
closest_point = ps.closest_point(p4)
# This should raise a proper error
# closest_point = my_library.Point2Collection([]).closest_point(p4)
print(f"Closest point: {closest_point}")

# print(type(ps))
# for p in ps.points:
#     print(type(p))

# Test numpy arrays
my_list = [0, 1, 2, 3, 4]
my_array = np.asarray([0, 1, 2, 3, 4, 5]).astype(float)
my_array2 = np.asarray([0, 1, 2, 3, 4, 5]).astype(float)
# TODO: read numpy arrays in rust functions / structs
my_array3 = my_library.mult_with_return_py(5, my_array2)
my_library.mult_mutable_py(5, my_array)
print(my_array)
print(my_array3)
assert my_array == my_array3
