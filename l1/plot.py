import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from shapely.geometry.polygon import LinearRing, Polygon
import os
import pickle

sns.set_theme(style="darkgrid")

df = pd.read_csv("weights.csv", delimiter=";")
df = df.sort_values(by=["map"])
#print(df)
sns.lineplot(x='map', y='value', hue="variable", data=pd.melt(df, ['map']))
plt.title("Weights")
plt.savefig("./plots/weights.png")

directory = './routes'
data_files = os.listdir(directory)
plt.clf()
for filename in data_files:
    if filename.endswith('.bin'):
        route = pickle.load(open(os.path.join(directory, filename), "rb"))
        poly = Polygon(route)
        x,y = poly.exterior.xy

        fig = plt.figure(1, figsize=(5,5), dpi=90)
        ax = fig.add_subplot(111)
        ax.plot(x, y)
        ax.set_title(f'{filename[:len(filename) - 4]}')
        plt.savefig(f"./plots/{filename[:len(filename) - 4]}.png")
        plt.clf()
        #print(route)

