import pandas as pd
import numpy as np
import matplotlib as mpl
mpl.use('Agg')
import matplotlib.pyplot as plt
import datetime

def print_full(x):
    pd.set_option('display.max_rows', len(x))
    print(x)
    pd.reset_option('display.max_rows')

from pandas import read_csv
series = read_csv('/python/landed-completed-IT-2017.csv', header=0, parse_dates=[0], index_col=0, squeeze=True)

#print(series.resample('1H', how='mean'))
print_full(series['2017-04'].rolling(40).mean().dropna()[::40])

plt.grid(True)
fig = plt.figure(figsize=(20,10))
ax = fig.add_subplot(111)
ax.plot(series.resample('1H', how='mean'))
fig.savefig('/python/plot-tm.png')

fig = plt.figure(figsize=(20,10))
ax = fig.add_subplot(111)
ax.plot(series.rolling(200).mean())
fig.savefig('/python/plot-wb.png')

fig = plt.figure(figsize=(60,10))
ax = fig.add_subplot(111)
ax.plot(series['2017-04-05'].rolling(100, min_periods = 10).mean().dropna()[::100])
fig.savefig('/python/plot-wb2.png')

# s = pd.Series([1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0])
# print_full(s.rolling(4).mean().dropna()[::2])
