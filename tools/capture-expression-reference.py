#!/usr/bin/env python3
"""Development-only capture; normal Rust tests need no Python interpreter."""
from pathlib import Path
import json, sys, dataclasses, struct, subprocess
import argparse, importlib.metadata, ctypes
parser=argparse.ArgumentParser(description="Capture pinned Python expression behaviour for Rust-only tests")
parser.add_argument("reference",type=Path)
parser.add_argument("--check",action="store_true")
args=parser.parse_args()
project_root=Path(__file__).resolve().parent.parent
reference=args.reference.resolve()
if sys.implementation.name!='cpython' or sys.version_info[:3]!=(3,12,14) or ctypes.sizeof(ctypes.c_long)!=8: raise SystemExit("Expression reference requires CPython 3.12.14 with 64-bit C long")
revision=subprocess.check_output(["git","-C",str(reference),"rev-parse","HEAD"],text=True).strip()
if revision!="e2d6f346a37f92483b9f9a13923b98feb053447b": raise SystemExit("Reference revision does not match the source lock")
if subprocess.check_output(["git","-C",str(reference),"status","--porcelain","--","src/eal","grammar"],text=True): raise SystemExit("Reference production sources are modified")
if importlib.metadata.version("antlr4-python3-runtime")!="4.13.2": raise SystemExit("Reference requires antlr4-python3-runtime 4.13.2")
sys.path.insert(0,str(reference/"src"))
from eal.parser import parse as parse_program
from eal.expressions import predicate_expression,evaluate_expression,expression_errors,format_expression,Quantity
cases=[]
def add(expression,context=None,schema=None):
 p=parse_program('language "EAL/3"\nenvironment e {\nrequire '+expression+'\n}\n')
 ast=predicate_expression(p.environments['e'].predicates[0])
 try:
  result=evaluate_expression(ast,context or {})
  exception=None
 except Exception as exc:
  from eal.expressions import ExpressionResult
  result=ExpressionResult(reason=str(exc),issue='invalid_expression')
  exception=type(exc).__name__
 def norm(value):
  if value is None:return {'type':'null'}
  if type(value)is bool:return {'type':'boolean','value':value}
  if type(value)is int:return {'type':'integer','value':str(value)}
  if type(value)is float:return {'type':'float','bits':struct.pack('>d',value).hex()}
  if type(value)is str:return {'type':'string','value':value}
  if type(value)is list:return {'type':'array','value':[norm(x)for x in value]}
  if type(value)is dict:return {'type':'object','value':{k:norm(v)for k,v in value.items()}}
  if isinstance(value,Quantity):return {'type':'quantity','numerator':str(value.amount.numerator),'denominator':str(value.amount.denominator),'dimensions':dict(value.dimensions)}
  raise ValueError(type(value))
 cases.append({'source':expression,'expression':dataclasses.asdict(ast),'context':context or {},'schema':schema,'result':{'known':result.known,'value':norm(result.value)if result.known else None,'reason':result.reason,'issue':result.issue},'errors':expression_errors(ast,schema),'formatted':format_expression(ast),'python_exception':exception})
for e in ['1+2*3 == 7','5 / -2','1 / 10','0 / -2','-5 % 2','5 % -2','-5 % -2','-5.0 % 2.0','5.0 % -2.0','-0.0 % -2.0','1.0 % -1e100','1e308+1e308','1 / 0','1.0 / 0','true == 1','true < false','null == null','null < null','[1] == [1]','{"a":1} == {"a":1}','not 1','+true','not false','-(-2)','sum([1,2,3])','sum([1e16,1,-1e16])','count([])','all([])','any([])','all([true,false])','any([true,false])','all([true,1])','sum([true])','min([3,1,2])','max(1,2,3)','min([])','max([true])','abs(-3)','abs(-0.0)','literal({"a": [1,true]})','wat(1)','count(1)','sum()','field("nested.value") == 2','field("missing") == 1','false and missing','true or missing','missing and false','missing or true','false and (1 / 0 == 2)','true or (1 / 0 == 2)','false and 1','true and missing','missing and (1 / 0 == 2)','quantity(1,"kPa") == quantity(1000,"Pa")','quantity(1,"ms") + quantity(1,"s") == quantity(1001,"ms")','quantity(36,"km/h") == quantity(10,"m/s")','quantity(0.1,"m")+quantity(0.2,"m") == quantity(0.3,"m")','0.1+0.2 == 0.3','quantity(1,"m") + quantity(1,"s") == quantity(2,"m")','quantity(2,"m")*quantity(3,"m")','quantity(1,"m")/quantity(2,"s") == quantity(0.5,"m/s")','quantity(1,"m") == 1','quantity(1,"m") < quantity(2,"s")','quantity(1,"unknown")','quantity(true,"m")','quantity(1,unit)','min(quantity(1,"kPa"),quantity(999,"Pa"))','max(quantity(1,"m"),quantity(1,"s"))','abs(quantity(-1,"m"))','-quantity(1,"m")','quantity(-5,"m") % quantity(2,"m")','quantity(1,"m") / quantity(0,"s")']:
 add(e,{'nested':{'value':2},'unit':'m'})
for e in ['9007199254740993 == 9007199254740992.0','9007199254740993 > 9007199254740992.0',('9'*80)+'+1',('1'+'0'*400)+' / '+('1'+'0'*400),('1'+'0'*400)+' * 0.0']:
 add(e)
add('field("𝄞.λ") == 2',{'𝄞':{'λ':2}})
add('field("🫨") == 2')
add('literal([-0, -0.0])')
add('sum([1e308,1e308])')
add('sum([1e308,1e308,-1e308,-1e308])')
for values in [[1e16,1.0,-1e16],[1e16,1,1.0,-1e16],[1e16,1.0,1,-1e16],[1,1e16,1.0,-1e16],[1e16,1.0,-1e16,1],[1.0,10**100,1.0,-10**100],[2**63-1,1,1e16,1.0,-1e16],[-2**63,-1,1e16,1.0,-1e16],[1e16,1.0,2**63,-2**63,-1e16],[1e16,1.0,-2**63-1,2**63+1,-1e16],[1e308,1e308,10**400],[-0.0],[-0.0,-0.0]]:
 add('sum('+json.dumps(values)+')')
add('field("x\\u200b") == 2')
add('field("x\\ue000") == 2')
add('value > 1',schema={'type':'object','properties':{'value':{'type':'string'}}})
add('value > 1',schema={'type':'object','properties':{}})
add('count(value) > 1',schema={'type':'object','properties':{'value':{'type':'array','items':{'type':'number'}}}})
add('quantity(value,"m") > quantity(0,"s")',schema={'type':'object','properties':{'value':{'type':'number'}}})
# Integer division must round the ratio without first converting huge operands.
for left,right in [(10**400,1),(1,10**400),(-1,10**400),(10**400,3),(2**1024-1,1),(2**1023,1),(10**400,10**400+10**384),(10**400,10**400-10**384)]:
 add(f'({left}) / ({right})')
# Deterministic adversarial numerical pairs exercise exact int/float boundaries.
import random
random_source=random.Random(6173)
integers=[0,1,-1,2**53-1,2**53,2**53+1,-2**53-1,10**80,-10**80,10**400,-10**400]
floats=[0.0,-0.0,0.1,-0.1,1.0,-1.0,1e-300,-1e-300,5e-324,-5e-324,1e100,-1e100,1e308,-1e308]
for index in range(160):
 left=random_source.choice(integers+floats)
 right=random_source.choice(integers+floats)
 operator=random_source.choice(['+','-','*','/','%','==','!=','<','<=','>','>='])
 add(f'({left!r}) {operator} ({right!r})')
sum_values=integers+floats+[2**63-1,2**63,-2**63,-2**63-1,1e16,-1e16]
for index in range(80):
 values=[random_source.choice(sum_values)for _ in range(random_source.randrange(0,16))]
 add('sum('+json.dumps(values)+')')
out={'schema':'eal-expression-reference/1','source_revision':revision,'python_version':sys.version.split()[0],'python_c_long_bits':ctypes.sizeof(ctypes.c_long)*8,'cases':cases}
destination=project_root/"fixtures/expressions/reference.json"
rendered=json.dumps(out,ensure_ascii=False,indent=2,allow_nan=False)+"\n"
if args.check:
 if destination.read_text()!=rendered: raise SystemExit("Expression reference has drifted")
else:
 destination.parent.mkdir(parents=True,exist_ok=True)
 destination.write_text(rendered)
print(f"Expression reference matches {len(cases)} cases" if args.check else f"Captured {len(cases)} expression cases")
