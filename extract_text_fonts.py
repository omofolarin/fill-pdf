#!/usr/bin/env python3
import sys
import json
from PyPDF2 import PdfReader

def extract_text_fonts(pdf_path):
    """Extract font information from actual text content in PDF pages"""
    try:
        reader = PdfReader(pdf_path)
        result = {'pages': [], 'fonts_used': set()}
        
        for page_num, page in enumerate(reader.pages):
            page_info = {
                'page': page_num + 1,
                'fonts': []
            }
            
            # Get page content stream
            if '/Contents' in page:
                content_ref = page['/Contents']
                content_obj = content_ref.get_object() if hasattr(content_ref, 'get_object') else content_ref
                
                # Handle array of content streams
                if isinstance(content_obj, list):
                    content_streams = content_obj
                else:
                    content_streams = [content_obj]
                
                for stream in content_streams:
                    if hasattr(stream, 'get_data'):
                        data = stream.get_data().decode('latin-1', errors='ignore')
                        
                        # Parse content stream for font operations
                        # Format: /FontName FontSize Tf
                        lines = data.split('\n')
                        for line in lines:
                            parts = line.strip().split()
                            for i, part in enumerate(parts):
                                if part == 'Tf' and i >= 2:
                                    try:
                                        font_name = parts[i-2]
                                        font_size = float(parts[i-1])
                                        font_info = {
                                            'font': font_name,
                                            'size': font_size
                                        }
                                        page_info['fonts'].append(font_info)
                                        result['fonts_used'].add(f"{font_name}@{font_size}pt")
                                    except (ValueError, IndexError):
                                        pass
            
            if page_info['fonts']:
                result['pages'].append(page_info)
        
        # Convert set to list for JSON serialization
        result['fonts_used'] = sorted(list(result['fonts_used']))
        
        return result
    except Exception as e:
        return {'error': str(e), 'pages': []}

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 extract_text_fonts.py <pdf_path>")
        sys.exit(1)
    print(json.dumps(extract_text_fonts(sys.argv[1]), indent=2))
