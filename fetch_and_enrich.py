import fetch_jira_metadata
import urllib.parse
import json
import random
import datetime

def main():
    username, token, base, project = fetch_jira_metadata.get_config()
    
    # Query for all issues, since we only want to pick up our prototype data
    jql = f'project = {project} AND issuetype IN (Epic, Task)'
    search_url = f"{base}/rest/api/3/search/jql"
    
    payload = {
        "jql": jql,
        "maxResults": 100,
        "fields": ["summary", "issuetype", "parent", "labels", "status"]
    }
    res = fetch_jira_metadata.make_request("POST", search_url, username, token, data=payload)
    if not res or 'issues' not in res:
        print("Failed to fetch data")
        return
        
    issues = res['issues']
    print(f"Fetched {len(issues)} raw issues.")
    
    # Filter our prototype testing epics and their children
    test_epics = {i['id']: i for i in issues if i['fields'].get('issuetype', {}).get('name') == 'Epic' and 'Prototyping Epic' in i['fields'].get('summary', '')}
    
    test_tasks = []
    for i in issues:
        if i['fields'].get('issuetype', {}).get('name') == 'Task':
            parent_id = None
            if i['fields'].get('parent'):
                parent_id = i['fields']['parent'].get('id')
            if parent_id in test_epics:
                test_tasks.append(i)
                
    filtered_issues = list(test_epics.values()) + test_tasks
    print(f"Filtered down to {len(test_epics)} Epics and {len(test_tasks)} Tasks.")
    
    # Add random mock dates
    today = datetime.date.today()
    
    for i in filtered_issues:
        if i['fields'].get('issuetype', {}).get('name') == 'Epic':
            start_offset = random.randint(-10, 5)
            duration = random.randint(10, 25)
            
            start_date = today + datetime.timedelta(days=start_offset)
            due_date = start_date + datetime.timedelta(days=duration)
            
            # Injecting them directly into fields object safely
            i['fields']['mockStartDate'] = start_date.isoformat()
            i['fields']['mockDueDate'] = due_date.isoformat()
            
    # Dump to JSON
    with open('jiri_data.json', 'w') as f:
        json.dump({"issues": filtered_issues}, f, indent=2)
        
    print("Successfully wrote jiri_data.json with mocked dates!")
    
if __name__ == "__main__":
    main()
