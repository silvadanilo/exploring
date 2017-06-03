import pandas as pd
import numpy as np
import datetime as dt
import time as tm

df = pd.read_csv('/tmp/python/exploring/projection-purchase-conversion-rate-2017.csv', dtype={"testing_group": str})
df['landed_at'] = pd.to_datetime(df['container_id'].str[0:8].apply(lambda x: int(x, 16)).apply(lambda x: dt.datetime.fromtimestamp(x).strftime('%Y-%m-%d %H:%M:%S')))
df = df.sort_values('landed_at')
df = df.set_index('landed_at')
df.fillna(value=0, inplace = True)
df = df.applymap(lambda x: 1 if x == 't' else x)
it_badoo = df[(df['country'] == 'IT') & (df['merchant'] == 'david@corp.badoo.com')]
# x.rolling(100, min_periods = 10).mean().dropna()[::100]
print(x.rolling(2, min_periods = 2).mean().dropna()[::2])


#print_full(df.where((df['opted_in'] == 1) & (df['country'] == 'IT')).groupby('merchant').agg({'completed': np.average}).sort('completed'))
