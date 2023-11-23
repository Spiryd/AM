import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

sns.set_theme(style="darkgrid")

df =  pd.read_csv("ls.csv", delimiter=";")
print(df)

plt.title("Mean Weights")
sns.lineplot(x='map', y='value', hue="variable", data=pd.melt(df[["map", "mst_weight", "dfs_mean", "random_mean", "mod_random_mean"]], ['map']))
plt.savefig("./plots/mean_weights.png")
plt.clf()

plt.title("Steps")
sns.lineplot(x='map', y='value', hue="variable", data=pd.melt(df[["map", "dfs_steps", "random_steps", "mod_random_steps"]], ['map']))
plt.savefig("./plots/steps.png")
plt.clf()

plt.title("Min Weight")
sns.lineplot(x='map', y='value', hue="variable", data=pd.melt(df[["map", "mst_weight", "dfs_min", "random_min", "mod_random_min"]], ['map']))
plt.savefig("./plots/min_weight.png")
plt.clf()
