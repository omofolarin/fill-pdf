#!/usr/bin/env python3
import sys
import json
from PyPDF2 import PdfReader

def extract_font_info(pdf_path):
    try:
        reader = PdfReader(pdf_path)
        result = {'fields': [], 'default_font_size': None}
        
        # Resolve root catalog
        root_ref = reader.trailer.get('/Root')
        root = root_ref.get_object() if hasattr(root_ref, 'get_object') else root_ref
        
        # Check AcroForm
        if root and '/AcroForm' in root:
            acro_ref = root['/AcroForm']
            acro = acro_ref.get_object() if hasattr(acro_ref, 'get_object') else acro_ref
            
            if '/DA' in acro:
                da = str(acro['/DA'])
                parts = da.split()
                for i, part in enumerate(parts):
                    if part == 'Tf' and i > 0:
                        try:
                            result['default_font_size'] = float(parts[i-1])
                        except:
                            pass
        
        # Get fields
        fields = reader.get_fields() if hasattr(reader, 'get_fields') else {}
        for name, obj in fields.items():
            info = {'name': name}
            if '/DA' in obj:
                da = str(obj['/DA'])
                parts = da.split()
                for i, part in enumerate(parts):
                    if part == 'Tf' and i > 0:
                        try:
                            info['font_size'] = float(parts[i-1])
                        except:
                            pass
            result['fields'].append(info)
        
        return result
    except Exception as e:
        return {'error': str(e), 'fields': []}

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 extract_font_info.py <pdf_path>")
        sys.exit(1)
    print(json.dumps(extract_font_info(sys.argv[1]), indent=2))
