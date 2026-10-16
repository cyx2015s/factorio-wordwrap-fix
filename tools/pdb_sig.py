
import ctypes as C, sys, os
from ctypes import wintypes
dbghelp = C.windll.dbghelp
SYMOPT_UNDNAME = 0x00000002
H = C.c_void_p(0x1000)
LOAD_BASE = 0x10000000
LOAD_SIZE = 0x08000000
class SYMBOL_INFO(C.Structure):
    _fields_ = [("SizeOfStruct", wintypes.ULONG), ("TypeIndex", wintypes.ULONG),
                ("Reserved", C.c_ulonglong*2), ("Index", wintypes.ULONG), ("Size", wintypes.ULONG),
                ("ModBase", C.c_ulonglong), ("Flags", wintypes.ULONG), ("Value", C.c_ulonglong),
                ("Address", C.c_ulonglong), ("Register", wintypes.ULONG), ("Scope", wintypes.ULONG),
                ("Tag", wintypes.ULONG), ("NameLen", wintypes.ULONG), ("MaxNameLen", wintypes.ULONG), ("Name", C.c_char*1)]
NAME_OFF = SYMBOL_INFO.Name.offset
CB = C.WINFUNCTYPE(wintypes.BOOL, C.POINTER(SYMBOL_INFO), wintypes.ULONG, C.c_void_p)
dbghelp.SymLoadModuleEx.argtypes = [H, H, C.c_char_p, C.c_char_p, C.c_ulonglong, wintypes.DWORD, C.c_void_p, wintypes.DWORD]
dbghelp.SymLoadModuleEx.restype = C.c_ulonglong
dbghelp.SymEnumSymbols.argtypes = [H, C.c_ulonglong, C.c_char_p, CB, C.c_void_p]
dbghelp.SymEnumSymbols.restype = wintypes.BOOL
u_dec = dbghelp.UnDecorateSymbolName
u_dec.argtypes = [C.c_char_p, C.c_char_p, wintypes.ULONG, wintypes.ULONG]
u_dec.restype = wintypes.ULONG
def load(pdb):
    dbghelp.SymSetOptions(dbghelp.SymGetOptions() & ~SYMOPT_UNDNAME)
    if not dbghelp.SymInitialize(H, None, False): raise OSError('SymInitialize failed')
    for img in (pdb, os.path.join(os.path.dirname(pdb), 'factorio.exe')):
        if os.path.exists(img):
            base = dbghelp.SymLoadModuleEx(H, None, img.encode(), None, LOAD_BASE, LOAD_SIZE, None, 0)
            if base: return base
    raise OSError('SymLoadModuleEx failed')
def demangle(name):
    if not name.startswith('?'): return name
    buf = C.create_string_buffer(2048)
    n = u_dec(name.encode(), buf, 2048, 0)
    return buf.value.decode('ascii', 'replace') if n else name
def main(pdb, term):
    base = load(pdb)
    found = []
    def collect(psym, size, ctx):
        info = psym.contents
        name = C.string_at(C.cast(psym, C.c_void_p).value + NAME_OFF, info.NameLen).decode('ascii','replace')
        if term in name: found.append(name)
        return True
    dbghelp.SymEnumSymbols(H, base, ('*'+term+'*').encode(), CB(collect), None)
    for name in sorted(set(found)):
        print(name)
        print('   -> ' + demangle(name))
        print()
if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
