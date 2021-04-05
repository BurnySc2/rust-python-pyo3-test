import time
from math import factorial
import numpy as np
import my_library


class Point2:
    def __init__(self, x, y):
        self.x = x
        self.y = y


def main():
    basic_tests()
    pathfinding_tests()
    pass


def pathfinding_tests():
    from plot_automaton import get_map_specific_bots
    from sc2.bot_ai import BotAI

    bot_object_generator = get_map_specific_bots()
    for i in bot_object_generator:
        print(i.game_info.map_name)
        if "automaton" in i.game_info.map_name.lower():
            random_bot_object = i
            break

    data = random_bot_object.game_info.pathing_grid.data_numpy
    data.ravel()
    # data.flat
    # data.flatten()

    width = 184
    height = 192

    # point_test = my_library.Point3d(3, 4, 5)
    # print(point_test)
    pass


def basic_tests():
    # Test basic library functions
    assert my_library.add_one(5) == 6
    assert my_library.add_one_and_a_half(5) == 6.5
    assert my_library.concatenate_string("hello") == "hello world!"
    # List
    assert my_library.sum_of_list([1, 2, 3]) == 6
    my_list = [1, 2, 3]
    my_library.append_to_list(my_list)
    assert my_list == [1, 2, 3, 420], my_list
    assert my_library.double_of_list([1, 2, 3]) == [2, 4, 6]
    my_dict = {}
    my_library.add_key_to_dict(my_dict)
    # Tuple
    assert my_library.tuple_interaction((5, 6)) == (5, 6, 11)
    # Dict
    assert my_dict == {"test": "hello"}, my_dict
    my_dict = {"hello": 5}
    my_library.change_key_value(my_dict)
    assert my_dict == {"hello": 6}, my_dict
    assert my_library.change_key_value_with_return(my_dict) == {"hello": 7}, my_dict
    # Set
    my_set = {1, 2, 3}
    my_library.add_element_to_set(my_set)
    assert my_set == {1, 2, 3, 420}, my_set
    assert my_library.add_element_to_set_with_return(my_set) == {1, 2, 3, 420, 421}, my_set

    # Big numbers
    a = factorial(400)
    b = my_library.big_num_factorial(400)
    assert str(a) == b, (a, b)


    # Numpy arrays
    # Have to convert to int64 because the default type is platform dependent
    # My guess is: 32bit machines will have int32 by default, and equivalent for 64bit machines
    array_1d = np.array([1, 2, 3]).astype(np.int64)
    array_2d = np.array([[1, 2, 3], [4, 5, 6]]).astype(np.int64)
    array_3d = np.array(
        [
            [[1, 2, 3], [4, 5, 6]],
            [[7, 8, 9], [10, 11, 12]],
        ]
    ).astype(np.int64)

    # Only 2d array allowed
    my_array = array_1d.copy()
    try:
        my_library.numpy_add_value_2d(my_array, 2)
        assert False, "Function above should throw error on 1d array"
    except TypeError:
        pass

    my_array = array_2d.copy()
    my_library.numpy_add_value_2d(my_array, 2)
    assert np.array_equal(my_array, array_2d.copy() + 2)

    # Adding number to any dimensional array is allowed
    my_array = array_1d.copy()
    my_library.numpy_add_value(my_array, 2)
    assert np.array_equal(my_array, array_1d.copy() + 2)
    my_array = array_2d.copy()
    my_library.numpy_add_value(my_array, 2)
    assert np.array_equal(my_array, array_2d.copy() + 2)

    # Check if any dimensions is possible
    assert my_library.numpy_calc_sum_of_array(array_1d.copy()) == array_1d.sum()
    assert my_library.numpy_calc_sum_of_array(array_2d.copy()) == array_2d.sum()
    assert my_library.numpy_calc_sum_of_array(array_3d.copy()) == array_3d.sum()

    # Test rust structs
    p1 = my_library.RustPoint2(0, 0)
    print(p1)
    p2 = my_library.RustPoint2(3, 4)
    print(p2)
    dist = p1.distance_to(p2)
    print(dist)
    dist_squared = p1.distance_to_squared(p2)
    print(dist_squared)

    ps = my_library.PointCollection([p1, p2])
    print(f"Amount of points in the list: {ps.len()}")
    # Add a Python Point2 point to the list of points
    p3 = my_library.RustPoint2(7, 8)
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

    p4 = my_library.RustPoint2(9, 10)
    closest_point = ps.closest_point(p4)
    # This should raise a proper error
    # closest_point = my_library.Point2Collection([]).closest_point(p4)
    print(f"Closest point: {closest_point}")

    print(ps, type(ps))
    for p in ps.points:
        print(p, type(p))


if __name__ == "__main__":
    main()
