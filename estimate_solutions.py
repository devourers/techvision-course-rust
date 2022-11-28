#99 168 156 0.4339 0.52829 0.797443 0.598173 0.6389 0.647 0.817264 0.5837 0.8787

SIZE = 300
DIAG = (SIZE*9*SIZE + SIZE*9*SIZE)**0.5

def eucl_dist(x, y):
    return ((x[0] - y[0])**2 + (x[1] - y[1])**2)**0.5

def parse_sol(sol):
    arr = sol.strip().split()
    x = int(arr[0])
    y = int(arr[1])
    h = int(arr[2])
    albedo = [float(a) for a in arr[3:]]
    return [x, y, h, albedo]

def prety_print_sol(sol):
    print("loc: ", [sol[0], sol[1]])
    print("height: ", sol[2] / DIAG)
    max_albedo = (max(sol[3]))
    for i in range(len(sol[3])):
        res_str = "albedo" + str(i) + "/albedo_max: " + str(sol[3][i] / max_albedo)
        print(res_str)


def compare_sols(sol1, sol2):
    loc1 = [sol1[0], sol1[1]]
    loc2 = [sol2[0], sol2[1]]
    print("error in loc: ", eucl_dist(loc1, loc2) / DIAG)
    print("error in height: ", abs(sol1[2] - sol2[2]) / DIAG)
    print("-----")
    max_albedo1 = max(sol1[3])
    max_albedo2 = max(sol2[3])
    for i in range(len(sol1[3])):
        res_str = "error in albedo" + str(i+1) + ": " + str(abs(sol2[3][i]/max_albedo2 -  sol1[3][i] / max_albedo1))
        print(res_str)


if __name__ == "__main__":
    sol1 = "0 899 388 0.46093848 0.55418356 0.72703212 0.60655967 0.64535064 0.5192756892 0.7895957606 0.49648596 0.61703406"
    sol2 = "0 899 387 0.5710 0.6840 0.8936 0.7646 0.8089 0.6404 1.0000 0.6245 0.7684"
    sol1 = parse_sol(sol1)
    sol2 = parse_sol(sol2)
    print("sol1")
    prety_print_sol(sol1)
    print("sol2")
    prety_print_sol(sol2)
    compare_sols(sol1, sol2)