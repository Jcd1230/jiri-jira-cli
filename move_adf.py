import json
import sys

def move_node(data):
    content = data.get('content', [])
    
    # 1. Find Quick Setup node
    quick_setup_index = -1
    for i, node in enumerate(content):
        # Look for the panel that contains the "Quick Setup" heading
        if node.get('type') == 'panel':
            panel_content = node.get('content', [])
            for subnode in panel_content:
                if subnode.get('type') == 'heading':
                    heading_content = subnode.get('content', [])
                    for t in heading_content:
                        if "Quick Setup for Developers" in t.get('text', ''):
                            quick_setup_index = i
                            break
                if quick_setup_index != -1: break
        if quick_setup_index != -1: break
            
    if quick_setup_index == -1:
        print("Quick Setup not found", file=sys.stderr)
        return None
        
    quick_setup_node = content.pop(quick_setup_index)
    
    # 2. Find Tech Stack node
    tech_stack_index = -1
    for i, node in enumerate(content):
        if node.get('type') == 'panel':
            panel_content = node.get('content', [])
            for subnode in panel_content:
                if subnode.get('type') == 'heading':
                    heading_content = subnode.get('content', [])
                    for t in heading_content:
                        if "Tech Stack" in t.get('text', ''):
                            tech_stack_index = i
                            break
                if tech_stack_index != -1: break
        if tech_stack_index != -1: break
            
    if tech_stack_index == -1:
        print("Tech Stack not found", file=sys.stderr)
        return None
        
    # 3. Insert before Tech Stack
    content.insert(tech_stack_index, quick_setup_node)
    
    data['content'] = content
    return data

if __name__ == "__main__":
    input_text = sys.stdin.read()
    
    # Find the start of the JSON
    # 'cargo run -- confluence view 5348294741 --raw' output:
    # Developer Tools and Setup (ID: 5348294741, Space: 4962091146, Version: 23)
    # =========================
    #
    # { ... }
    
    lines = input_text.splitlines()
    json_lines = []
    in_json = False
    for line in lines:
        if line.startswith('{'):
            in_json = True
        if in_json:
            json_lines.append(line)
            
    if not json_lines:
        print("Could not find start of JSON", file=sys.stderr)
        sys.exit(1)
        
    json_text = "\n".join(json_lines)
    data = json.loads(json_text)
    modified_data = move_node(data)
    if modified_data:
        print(json.dumps(modified_data))
