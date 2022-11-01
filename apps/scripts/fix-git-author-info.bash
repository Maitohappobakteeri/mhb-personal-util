git filter-branch --env-filter '
if [ "$GIT_AUTHOR_EMAIL" = "old-email-here!!!" ]; then
    GIT_AUTHOR_EMAIL="maitohappobakteeri@proton.me";
    GIT_AUTHOR_NAME="Jenna";
    GIT_COMMITTER_EMAIL=$GIT_AUTHOR_EMAIL;
    GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME"; 
fi' -- --all