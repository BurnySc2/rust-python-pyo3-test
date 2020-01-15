import sys
import os
import lzma
import pickle


import numpy as np
np.set_printoptions(threshold=sys.maxsize)
import matplotlib.pyplot as plt

from typing import Tuple, List, Iterable

from sc2.game_data import GameData
from sc2.game_info import GameInfo
from sc2.game_state import GameState
from sc2.bot_ai import BotAI

def get_map_specific_bots() -> Iterable[BotAI]:
    folder = os.path.dirname(__file__)
    subfolder_name = folder
    # subfolder_name = "pickle_data"
    pickle_folder_path = os.path.join(folder, subfolder_name)
    files = os.listdir(pickle_folder_path)
    for file in (f for f in files if f.endswith(".xz")):
        with lzma.open(os.path.join(folder, subfolder_name, file), "rb") as f:
            raw_game_data, raw_game_info, raw_observation = pickle.load(f)

        # Build fresh bot object, and load the pickle'd data into the bot object
        bot = BotAI()
        game_data = GameData(raw_game_data.data)
        game_info = GameInfo(raw_game_info.game_info)
        game_state = GameState(raw_observation)
        bot._initialize_variables()
        bot._prepare_start(client=None, player_id=1, game_info=game_info, game_data=game_data)
        bot._prepare_step(state=game_state, proto_game_info=raw_game_info)

        yield bot
# Global bot object that is used in TestClass.test_position_*
bot_object_generator = get_map_specific_bots()
random_bot_object: BotAI = next(bot_object_generator)

# print(random_bot_object.game_info.start_locations)
# print(random_bot_object.townhalls[0].position)
# print(random_bot_object.enemy_start_locations)

def main():
    start = (29, 65)
    goal = (154, 114)
    # start = (32, 51)
    # goal = (150, 129)
    # map_grid = np.loadtxt("AutomatonLE.txt", delimiter="").astype(int)
    grid = []
    with open("AutomatonLE.txt") as f:
        for line in f.readlines():
            values = [int(i) for i in list(line.strip())]
            grid.append(values)
    # print(grid)
    map_grid = np.asarray(grid)
    # print(map_grid)

    path = []
    with open("path.txt") as f:
        for line in f.readlines():
            x, y = line.split(",")
            path.append((int(x.strip()), int(y.strip())))
    print()
    # print(map_grid.shape)
    plot(map_grid, route=path, start=start, goal=goal)


def plot(
    grid, route: List[Tuple[int, int]] = None, start: Tuple[int, int] = None, goal: Tuple[int, int] = None, waypoints=None
):
    # extract x and y coordinates from route list
    x_coords = []
    y_coords = []
    if route:
        for i in range(0, len(route)):
            x = route[i][0]
            y = route[i][1]
            x_coords.append(x)
            y_coords.append(y)

    # plot map and path
    fig, ax = plt.subplots(figsize=(20, 20))
    ax.imshow(grid, cmap=plt.cm.Dark2)
    if start:
        ax.scatter(start[0], start[1], marker="x", color="red", s=200)
    if goal:
        ax.scatter(goal[0], goal[1], marker="x", color="blue", s=200)
    if route:
        for w in route:
            ax.scatter(w[0], w[1], marker="x", color="orange", s=100)

    if waypoints:
        for w in waypoints:
            ax.scatter(w[0], w[1], marker="x", color="black", s=50)
    # plt.gca().invert_xaxis()
    plt.gca().invert_yaxis()
    plt.show()


if __name__ == "__main__":
    main()
