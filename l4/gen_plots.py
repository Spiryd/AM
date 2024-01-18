import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

df = pd.read_csv('data.csv', sep=';')

sns.set_style('darkgrid')

sns.lineplot(data=df, x='map', y='avg_weight')
plt.savefig('weight.png')
plt.clf()

sns.lineplot(data=df, x='map', y='avg_time')
plt.savefig('time.png')
