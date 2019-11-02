import json
json_path = '/home/yasushi/code/liq/jp_cabinet_result.json'
out_path = '/home/yasushi/share/Projects/2019-11-02_Japan_Cabinet_WS_Result/result.csv'

result = []


with open(json_path, 'r') as file:
    data = json.load(file)

output = data['output'] 

# header for influence
inf = []
result.append("voter, influence")
for key in output['influence']:
    inf.append('{},{}'.format(key, output['influence'][key]))   

inf.sort()

result += inf

result.append("")
result.append("policy, popularity")
vts = []
for key in output['votes']:
    vts.append('{},{}'.format(key, output['votes'][key]))

vts.sort()
result += vts

with open(out_path, 'w') as out_file:
    out_file.writelines('{}\n'.format(l) for l in result)