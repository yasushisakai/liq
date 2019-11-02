import json
csv_path = '/home/yasushi/share/Projects/2019-11-02_Japan_Cabinet_WS_Result/pre_json.csv'

voters = []
votes = {}

with open(csv_path, 'r') as f:
    lines = f.readlines()
    tags = map(lambda s: s.strip(), lines[0].split(',')[1:])

    policies = map(lambda tag: {'Short': tag}, filter(lambda t: t.startswith('P'), tags));
    
    for l in lines[1:]:
        splitted = l.split(',')

        if not splitted[0] : 
           continue # MS word garbage

        voters.append(splitted[0])
        obj = dict(zip(tags, map(lambda x: float(x.strip()), splitted[1:])))
        votes[splitted[0]] = obj

data = {}

for o in ('voters','policies', 'votes'):
    data[o] = locals()[o]

# print(data)
print(json.dumps(data,sort_keys=True, indent=4, separators=(',', ': ')))
        