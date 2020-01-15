## Idea of Jump-Point-Search (JPS)

Each node consists of:
- (x, y): Start point
- (dx, dy): Direction
- float: Real cost to start
- float: Total cost estimate
- bool: There is a wall on the left
- bool: There is a wall on the right

Priority of node execution:

-
    Each node has a total cost: Real distance to start + estimated (diagonal distance) distance to goal
    
    Execute node with lowest total cost
    
- 
    Each node has a direction: non diagonal or diagonal
    
    Execute non diagonal first, then diagonal ones

## Explanation

While traversing non-diagonally, we only start moving diagonally after there was a wall on the left or right side of traversal direction.

In the picture (D), a new node is created with position (3, a) and direction (1, -1) and right_is_wall=True and left_is_wall=? (depends on if (4, A) is a wall).

![Horizontal and vertical scan](https://uploads.gamedev.net/monthly_10_2015/ccs-231493-0-42311400-1445766300.png)

While traversing diagonally, we start traversing non-diagonally while there is no wall in 45 degree angles.

In the picture (E), a new node is created with position (3, b) and direction (1, 0) and another node in (2, c) with direction (0, 1).

In the picture (G), additionally to creation of nodes in (E), another node is created in (1, c) with direction (-1, 1) and left_is_wall=True and right_is_wall=False (assume (c, 2) is not a wall).

![Diagonal scan](https://uploads.gamedev.net/monthly_10_2015/ccs-231493-0-79392600-1445766667.png)

