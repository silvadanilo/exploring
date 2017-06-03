import pandas as pd
import numpy as np
import matplotlib as mpl
mpl.use('Agg')
import matplotlib.pyplot as plt
import datetime

s = pd.Series([27.45,
6.9,
45,
42.59,
41.05,
33.59,
25.35,
34.12,
33.52,
30.77,
32.51,
32.91,
32.57,
35.23,
32.97,
33.99,
36.5,
38.73,
33.23,
30.91,
27.19,
7.69,
15.7,
35.23,
33.33,
15.09,
43.9,
42.62,
53.42,
32.77,
37.65,
37.58,
19.29,
37.06,
33.73,
30.24,
34.01,
35.95,
34.33,
39.25,
41.47,
39.56,
31.8,
32.6,
27.06,
35.85,
41.24,
23.08,
34.78,
6.25,
17.5,
12.5,
2.72,
32.16,
45.51,
29.95,
35.98,
27.31,
38.04,
33.79,
32.63,
28.39,
42.11,
41.05,
32.31,
35.43,
38.63,
34.81,
33.81,
32.93,
27.54,
31.94,
37.5,
16.85,
20.73,
40.35,
46.15,
38.79,
32.99,
34.11,
36.06,
25.49,
39.86,
37.2,
36.1,
34.98,
31.99,
29.69,
29.84,
35.21,
34.94,
25.37,
14.89,
18.23,
21.09,
16.04,
13.16,
12,
42,
24.44,
41.57,
38.69,
39.15,
31.92,
33.48,
36.95,
33.78,
32.27,
38.01,
38.71,
42.17,
33.63,
32.38,
30.22,
33.2,
28.26,
34.22,
28.26,
27.66,
26.92,
24,
8.82,
37.14,
44.9,
47.37,
34.86,
37.58,
32.29,
34.67,
39.91,
45.26,
45.15,
31.64,
37.4,
30.08,
33.33,
32.2,
35.96,
34.28,
28.01,
28.69,
32.68,
21.83,
32.69,
46.15,
20.83,
39.29,
41.51,
34.74,
34.97,
35.64,
40.46,
26.59,
31.78,
31.6,
33.57,
44.61,
30.93,
35.79,
31,
29.8,
30.75,
36.16,
30.98,
34.8,
22.75,
25.24,
25,
30.95,
19.44,
26.67,
44.64,
39.39,
41.38,
34.07,
21.15,
39.61,
32.47,
32.09,
42.62,
27.08,
37.9,
26.79,
40.08,
35.1,
34.05,
33.65,
36.06,
31.82,
27.59,
32.05,
40.32,
23.26,
11.11,
31.71,
33.33,
54.41,
43.48,
39.55,
37.86,
43.38,
34.13,
34.83,
30.36,
39.2,
32.41,
34.45,
37.88,
36.62,
37.8,
43.8,
30.45,
32.05,
33.12,
25.77,
34.92,
30.95,
8.51,
36.59,
38.3,
38.3,
33.08,
32.52,
26.16,
33.04,
31.35,
41,
35.19,
32.1,
40.95,
32.11,
39.32,
36.73,
37.12,
36.13,
33.16,
29.82,
33.55,
28.85,
31.71,
18.75,
11.96,
27.59,
44.59,
41.38,
37.01,
39.06,
42.55,
34.59,
51.58,
37.1,
36.24,
30.4,
32.5,
41.27,
37.25,
34.98,
41.55,
30.55,
39.3,
28.65,
33.77,
36.27,
35.82,
31.91,
18.92,
51.35,
37.78,
37.04,
46.02,
39.26,
35.9,
42.05,
47.37,
39.27,
39.82,
31.07,
33.47,
36.29,
37.98,
38.46,
33.21,
31.42,
39.73,
38.17,
26.81,
35.21,
24.53,
41.38,
13.33,
50,
28.81,
39.13,
48.15,
40.85,
31.46,
17.39,
35.78,
35.81,
41.21,
37.33,
41.45,
30.74,
39.92,
41.23,
38.76,
31.97,
33.57,
33.05,
34.16,
26.47,
35,
30.56,
19.05,
18.64,
13.64,
17.09,
41.48,
39.61,
41.26,
33.33,
36.31,
40.48,
36.2,
32.51,
30,
35.19,
39.45,
36.84,
39.15,
35.64,
34.92,
29.67,
30.19,
25.81,
35,
35.71,
12.2,
16.67,
18.56,
7.09,
3.55,
40,
35.33,
37.99,
41.07,
42.33,
34.86,
37.19,
36.32,
39.49,
35.96,
42.58,
32.03,
30.24,
27.51,
36.32,
22.82,
21,
24.1,
25.64,
10.26,
33.33,
37.5,
42.11,
35.77,
41.67,
26.79,
36.42,
37.76,
36.9,
46.15,
43.67,
38.64,
31.22,
34.68,
36.14,
34.11,
31.62,
32.92,
18.62,
24.03,
34.48,
22.22,
32.43,
14.29,
60,
53.49,
42.99,
43.93,
40,
39.49,
41.61,
41.92,
41.53,
42.36,
30.48,
30.69,
30.81,
38.76,
43.55,
35.93,
35.25,
31.28,
32.77,
34.59,
29.09,
27.16,
29.73,
14.47,
35.71,
32.86,
35.92,
38.74,
29.22,
36.59,
41.13,
37.36,
38.97,
37.07,
34.98,
34.23,
33.6,
32.39,
41.49,
38.35,
48.36,
30.97,
21.61,
33.12,
30.1,
22.03,
23.44,
7.87,
21.74,
51.92,
37.11,
40.85,
39.74,
36.93,
34.36,
30.1,
39.48,
24.46,
39.78,
32.84,
34.85,
29.96,
34.18,
34.68,
28.22,
26.79,
25.43,
22.02,
20,
23.08,
35.29,
3.23,
2.7,
1.11,
10.2,
40.31,
40,
35.08,
31.09,
34.58,
40.53,
31.14,
36.12,
31.56,
33.06,
36.62,
35.94,
31.87,
32.62,
37.41,
34.84,
28.17,
31.76,
39.47,
19.05,
12.2,
35.29,
33.33,
24.56,
30.77,
44.72,
32.69,
37.16,
27.73,
36.9,
38.5,
32.62,
32.95,
27.39,
34.65,
34.97,
42.13,
30.99,
29.34,
24.37,
5.21,
9.64,
20.83,
35.48,
8.16,
40.54,
31.94,
44.21,
44.26,
42.96,
33.33,
38.75,
34.15,
27.67,
31.33,
36.1,
38.26,
29.79,
24.13,
33.33,
35.4,
31.51,
34.3,
28.14,
31.01,
23.3,
18.03,
11.36,
17.78,
28.13,
24.32,
28.1,
32.81,
41.61,
36.09,
17.42,
37.24,
35.18,
37.17,
32.34,
24.88,
31.86,
31.65,
28.64,
30.26,
29.32,
31.4,
31.58,
35.62,
26.77,
26.15,
16.67,
7.14,
22.81,
22.22,
23.26,
40.15,
35.44,
29.7,
29.41,
30.97,
32.14,
23.32,
23.11,
33.56,
30.64,
39.11,
31.93,
33.7,
32.36,
36.24,
24,
32.14,
31.18,
36.92,
34.15,
8.62,
22.22,
22.95,
30,
29.29,
32.87,
42.77,
33.33,
32.18,
42.02,
35.24,
28.15,
23.26,
26.15,
31.78,
30.63,
36.92,
29.69,
25,
22.27,
11.79,
4.35,
22.76,
11.94,
20.45,
28.57,
33.67,
38.89,
38.85,
33.51,
34.65,
33.94,
34.85,
30.04,
29.18,
32.27,
30.92,
35.86,
30.9,
32.66,
35.16,
33.17,
20.43,
25.74,
30.91,
13.51,
9.09,
38.46,
25,
57.38,
43.94,
41.84,
35.91,
25.19,
32.42,
29.84,
28.63,
24.34,
26.61,
29.63,
35.63,
27.27,
33.98,
28.37,
30.65,
32.16,
26.4,
33,
17.91,
18.75,
8.33,
23.53,
41.18,
49.18,
47.73,
35.03,
29.5,
32.05,
28.65,
31.89,
28.36,
26.27,
30.32,
32.14,
30.04,
31.53,
31.98,
25.67,
32.76,
25.33,
28.13,
23.91,
18.64,
19.05,
16,
9.62,
29.31,
47.17,
40.17,
35.34,
35.58,
24.4,
33.55,
27.62,
33.87,
26,
28.31,
37.2,
30.4,
33.63,
40.54,
29.78,
31.65,
28.57,
28.91,
31.63,
18.46,
22.86,
6.9,
63.64,
27.27,
35.8,
33.64,
35.09,
32.79,
26.67,
35.51,
26.37,
28.34,
26.74,
29.84,
32.09,
28.78,
38.01,
31.93,
35.38,
31.76,
22.87,
30.16,
28.13,
32,
28.13
])

#print(s.mean())

#csv = pd.read_csv('/tmp/cr.csv', sep=',', header=0, parse_dates=[0])
#print(type(csv))

def dateparse (time_in_secs):
    return datetime.datetime.fromtimestamp(float(time_in_secs))

from pandas import read_csv
#series = read_csv('/tmp/cr2.csv', header=0, parse_dates=[0], index_col=0, squeeze=True, date_parser=dateparse)
series = pd.Series([2,3,2,4,2,3])
#print(series.resample("D", how='sum'))
print(series.describe())


fig = plt.figure(figsize=(20,10))
ax = fig.add_subplot(111)
ax.plot(series)
fig.savefig('/python/plot.png')

fig2 = plt.figure(figsize=(20,10))
ax2 = fig2.add_subplot(111)
ax2.plot(series.rolling(window=2,center=True).mean())
fig2.savefig('/python/plot2.png')
