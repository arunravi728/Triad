import json
import re
import sys
import os

def convert_jsonc_to_json(input_path):
    output_path = input_path.rsplit('.', 1)[0] + '.json'
    
    with open(input_path, 'r') as f:
        content = f.read()

    # 1. Remove multi-line comments: /* ... */
    content = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)
    
    # 2. Remove single-line comments: // ...
    content = re.sub(r'//.*', '', content)
    
    # 3. Remove trailing commas before closing braces/brackets
    content = re.sub(r',\s*([\]}])', r'\1', content)

    try:
        # Validate the resulting string is valid JSON
        data = json.loads(content)
        
        with open(output_path, 'w') as f:
            json.dump(data, f, indent=4)
            
        print(f"Successfully created: {output_path}")
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON after cleaning: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python jsonc_to_json.py <filename.jsonc>")
    else:
        convert_jsonc_to_json(sys.argv[1])