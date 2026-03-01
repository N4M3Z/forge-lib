# SKILL.md schema — forge module skills
# Requires: frontmatter identity, instructions/usage section, constraints

frontmatter:
    fields:
        - name: name
        - name: description
        - name: version

heading_rules:
    no_skip_levels: true
    max_depth: 3

structure:
    - heading:
        pattern: '# .*'
        regex: true
      allow_additional: true
      children:
        - heading:
            pattern: '## (Instructions|Usage)'
            regex: true
        - heading: '## Setup'
          optional: true
          allow_additional: true
        - heading: '## Constraints'
