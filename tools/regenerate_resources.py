#!/usr/bin/env python3
import glob, json, os

# folder path
images_path = 'images'




if __name__=='__main__':
    existing_resources = {
    'images':[]
    }
    try:
        with open("resources.json", "r") as f1:
            existing_resources = json.load(f1)
    except FileNotFoundError:
        pass
    resources_json = {}
    
    # Load images
    existing_ids = set()
    existing_imgs = set()
    for img in existing_resources['images']:
        existing_ids.add(img['id'])
        existing_imgs.add(img['path'])
    # list to store files
    images = []
    # Iterate directory
    for path in glob.glob('./'+images_path+'/*.ff'):
        # check if current path is a file
        if os.path.isfile(path):
            images.append(path)
    resources_json['images'] = existing_resources['images']
    for im in images:
        name = os.path.basename(im)
        path = im if im[0] != "." else im[1:]
        if path not in existing_imgs:
            im_id = 0
            while im_id in existing_ids:
                im_id+=1
            resources_json['images'].append({
                'id': im_id,
                'name': name,
                'path': path
            })
            existing_ids.add(im_id)
    # Done loading resources
    with open('resources.json', 'w') as f1:
        json.dump(resources_json, f1)
